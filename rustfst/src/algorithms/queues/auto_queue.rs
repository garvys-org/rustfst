use anyhow::Result;

use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::tr_filters::TrFilter;
use crate::algorithms::visitors::SccVisitor;
use crate::algorithms::{Queue, QueueType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::ExpandedFst;
use crate::semirings::{Semiring, SemiringProperties};

use super::{
    natural_less, FifoQueue, LifoQueue, NaturalShortestFirstQueue, SccQueue, StateOrderQueue,
    TopOrderQueue, TrivialQueue,
};
use crate::{StateId, Trs};

#[derive(Debug)]
pub struct AutoQueue {
    queue: Box<dyn Queue>,
}

impl AutoQueue {
    pub fn new<W: Semiring, F: ExpandedFst<W>, A: TrFilter<W>>(
        fst: &F,
        distance: Option<&Vec<W>>,
        tr_filter: &A,
    ) -> Result<Self> {
        let props = fst.properties();

        let queue: Box<dyn Queue>;

        if props.contains(FstProperties::TOP_SORTED) || fst.start().is_none() {
            queue = Box::<StateOrderQueue>::default();
        } else if props.contains(FstProperties::ACYCLIC) {
            queue = Box::new(TopOrderQueue::new(fst, tr_filter));
        } else if props.contains(FstProperties::UNWEIGHTED)
            && W::properties().contains(SemiringProperties::IDEMPOTENT)
        {
            queue = Box::<LifoQueue>::default();
        } else {
            let mut scc_visitor = SccVisitor::new(fst, true, false);
            dfs_visit(fst, &mut scc_visitor, tr_filter, false);
            let sccs: Vec<_> = scc_visitor
                .scc
                .unwrap()
                .into_iter()
                .map(|v| v as StateId)
                .collect();
            let n_sccs = scc_visitor.nscc as usize;

            let mut queue_types = vec![QueueType::TrivialQueue; n_sccs];
            let less = if distance.is_some()
                && !distance.unwrap().is_empty()
                && W::properties().contains(SemiringProperties::PATH)
            {
                Some(natural_less)
            } else {
                None
            };

            // Finds the queue type to use per SCC.
            let mut unweighted = false;
            let mut all_trivial = false;
            Self::scc_queue_type(
                fst,
                &sccs,
                less,
                &mut queue_types,
                &mut all_trivial,
                &mut unweighted,
                tr_filter,
            )?;

            if unweighted {
                // If unweighted and semiring is idempotent, uses LIFO queue.
                queue = Box::<LifoQueue>::default();
            } else if all_trivial {
                // If all the SCC are trivial, the FST is acyclic and the scc number gives
                // the topological order.
                queue = Box::new(TopOrderQueue::from_precomputed_order(sccs));
            } else {
                // AutoQueue: using SCC meta-discipline
                let mut queues: Vec<Box<dyn Queue>> = Vec::with_capacity(n_sccs);
                for queue_type in queue_types.iter().take(n_sccs) {
                    match queue_type {
                        QueueType::TrivialQueue => queues.push(Box::<TrivialQueue>::default()),
                        QueueType::ShortestFirstQueue => queues.push(Box::new(
                            NaturalShortestFirstQueue::new(distance.unwrap().clone()),
                        )),
                        QueueType::LifoQueue => queues.push(Box::<LifoQueue>::default()),
                        _ => queues.push(Box::<FifoQueue>::default()),
                    }
                }
                queue = Box::new(SccQueue::new(queues, sccs));
            }
        }

        Ok(Self { queue })
    }

    pub fn scc_queue_type<
        W: Semiring,
        F: ExpandedFst<W>,
        C: Fn(&W, &W) -> Result<bool>,
        A: TrFilter<W>,
    >(
        fst: &F,
        sccs: &[StateId],
        compare: Option<C>,
        queue_types: &mut [QueueType],
        all_trivial: &mut bool,
        unweighted: &mut bool,
        tr_filter: &A,
    ) -> Result<()> {
        *all_trivial = true;
        *unweighted = true;

        queue_types
            .iter_mut()
            .for_each(|v| *v = QueueType::TrivialQueue);

        for state in 0..(fst.num_states() as StateId) {
            for tr in unsafe { fst.get_trs_unchecked(state).trs() } {
                if !tr_filter.keep(tr) {
                    continue;
                }
                if sccs[state as usize] == sccs[tr.nextstate as usize] {
                    let queue_type =
                        unsafe { queue_types.get_unchecked_mut(sccs[state as usize] as usize) };
                    if compare.is_none() || compare.as_ref().unwrap()(&tr.weight, &W::one())? {
                        *queue_type = QueueType::FifoQueue;
                    } else if *queue_type == QueueType::TrivialQueue
                        || *queue_type == QueueType::LifoQueue
                    {
                        if !W::properties().contains(SemiringProperties::IDEMPOTENT)
                            || (!tr.weight.is_zero() && !tr.weight.is_one())
                        {
                            *queue_type = QueueType::ShortestFirstQueue;
                        } else {
                            *queue_type = QueueType::LifoQueue;
                        }
                    }

                    if *queue_type != QueueType::TrivialQueue {
                        *all_trivial = false;
                    }
                }

                if !W::properties().contains(SemiringProperties::IDEMPOTENT)
                    || (!tr.weight.is_zero() && !tr.weight.is_one())
                {
                    *unweighted = false;
                }
            }
        }
        Ok(())
    }
}

impl Queue for AutoQueue {
    fn head(&mut self) -> Option<StateId> {
        self.queue.head()
    }

    fn enqueue(&mut self, state: StateId) {
        self.queue.enqueue(state)
    }

    fn dequeue(&mut self) -> Option<StateId> {
        self.queue.dequeue()
    }

    fn update(&mut self, state: StateId) {
        self.queue.update(state)
    }

    fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    fn clear(&mut self) {
        self.queue.clear()
    }

    fn queue_type(&self) -> QueueType {
        QueueType::AutoQueue
    }
}
