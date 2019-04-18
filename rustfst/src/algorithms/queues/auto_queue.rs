use failure::Fallible;

use super::{
    natural_less, FifoQueue, LifoQueue, NaturalShortestFirstQueue, SccQueue, StateOrderQueue,
    TopOrderQueue, TrivialQueue,
};
use crate::algorithms::{find_strongly_connected_components, Queue, QueueType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{Semiring, SemiringProperties};

#[derive(Debug)]
pub struct AutoQueue {
    queue: Box<Queue>,
}

impl AutoQueue {
    pub fn new<F: MutableFst + ExpandedFst>(fst: &F, distance: Option<&Vec<F::W>>) -> Fallible<Self>
    where
        F::W: 'static,
    {
        let props = fst.properties()?;

        let mut queue: Box<Queue>;

        if props.contains(FstProperties::TOP_SORTED) || fst.start().is_none() {
            queue = Box::new(StateOrderQueue::new());
        } else if props.contains(FstProperties::ACYCLIC) {
            queue = Box::new(TopOrderQueue::new(fst));
        } else if props.contains(FstProperties::UNWEIGHTED)
            && F::W::properties().contains(SemiringProperties::IDEMPOTENT)
        {
            queue = Box::new(LifoQueue::new());
        } else {
            let mut sccs: Vec<usize> = vec![];
            let mut n_sccs: usize = 0;
            find_strongly_connected_components(fst, &mut sccs, &mut n_sccs)?;

            let mut queue_types = vec![QueueType::TrivialQueue; n_sccs];
            let mut less = None;
            if distance.is_some()
                && distance.unwrap().len() >= 1
                && F::W::properties().contains(SemiringProperties::PATH)
            {
                less = Some(natural_less);
            }
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
            )?;

            if unweighted {
                // If unweighted and semiring is idempotent, uses LIFO queue.
                queue = Box::new(LifoQueue::new());
            } else if all_trivial {
                // If all the SCC are trivial, the FST is acyclic and the scc number gives
                // the topological order.
                queue = Box::new(TopOrderQueue::from_precomputed_order(sccs));
            } else {
                // AutoQueue: using SCC meta-discipline
                let mut queues: Vec<Box<Queue>> = Vec::with_capacity(n_sccs);
                for i in 0..n_sccs {
                    match queue_types[i] {
                        QueueType::TrivialQueue => queues.push(Box::new(TrivialQueue::new())),
                        QueueType::ShortestFirstQueue => queues.push(Box::new(
                            NaturalShortestFirstQueue::new(distance.unwrap().clone()),
                        )),
                        QueueType::LifoQueue => queues.push(Box::new(LifoQueue::new())),
                        _ => queues.push(Box::new(FifoQueue::new())),
                    }
                }
                queue = Box::new(SccQueue::new(queues, sccs));
            }
        }

        Ok(Self { queue })
    }

    pub fn scc_queue_type<F: MutableFst + ExpandedFst, C: Fn(&F::W, &F::W) -> Fallible<bool>>(
        fst: &F,
        sccs: &Vec<usize>,
        compare: Option<C>,
        queue_types: &mut Vec<QueueType>,
        all_trivial: &mut bool,
        unweighted: &mut bool,
    ) -> Fallible<()> {
        *all_trivial = true;
        *unweighted = true;

        queue_types
            .iter_mut()
            .for_each(|v| *v = QueueType::TrivialQueue);

        let states: Vec<_> = fst.states_iter().collect();
        for state in states {
            for arc in fst.arcs_iter(state)? {
                if sccs[state] == sccs[arc.nextstate] {
                    let queue_type = unsafe { queue_types.get_unchecked_mut(sccs[state]) };
                    if compare.is_none() || compare.as_ref().unwrap()(&arc.weight, &F::W::one())? {
                        *queue_type = QueueType::FifoQueue;
                    } else if *queue_type == QueueType::TrivialQueue
                        || *queue_type == QueueType::LifoQueue
                    {
                        if !F::W::properties().contains(SemiringProperties::IDEMPOTENT)
                            || (!arc.weight.is_zero() && !arc.weight.is_one())
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

                if !F::W::properties().contains(SemiringProperties::IDEMPOTENT)
                    || (!arc.weight.is_zero() && !arc.weight.is_one())
                {
                    *unweighted = false;
                }
            }
        }
        Ok(())
    }
}

impl Queue for AutoQueue {
    fn head(&mut self) -> Option<usize> {
        self.queue.head()
    }

    fn enqueue(&mut self, state: usize) {
        self.queue.enqueue(state)
    }

    fn dequeue(&mut self) {
        self.queue.dequeue()
    }

    fn update(&mut self, state: usize) {
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
