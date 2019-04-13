use failure::Fallible;

use super::{LifoQueue, StateOrderQueue, TopOrderQueue};
use crate::algorithms::{find_strongly_connected_components, Queue, QueueType};
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, MutableFst};
use crate::semirings::{Semiring, SemiringProperties};

pub struct AutoQueue {
    queue: Box<Queue>,
}

impl AutoQueue {
    pub fn new<F: MutableFst + ExpandedFst>(fst: &F, distance: &Vec<F::W>) -> Fallible<Self> {
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

            let queue_types = vec![QueueType::TrivialQueue; n_sccs];

            unimplemented!()
        }

        unimplemented!()
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
                    let queue_type = unsafe {queue_types.get_unchecked_mut(sccs[state])};
                    if compare.is_none() || compare.unwrap()(&arc.weight, &F::W::one())? {
                        *queue_type = QueueType::FifoQueue;
                    } else if *queue_type == QueueType::TrivialQueue || *queue_type == QueueType::LifoQueue {
                        if !F::W::properties().contains(SemiringProperties::IDEMPOTENT) || (!arc.weight.is_zero() && !arc.weight.is_one()) {
                            *queue_type = QueueType::ShortestFirstQueue;
                        } else {
                            *queue_type = QueueType::LifoQueue;
                        }
                    }

                    if *queue_type != QueueType::TrivialQueue {*all_trivial = false;}
                }

                if !F::W::properties().contains(SemiringProperties::IDEMPOTENT) || (!arc.weight.is_zero() && !arc.weight.is_one()) {
                    *unweighted = false;
                }
            }
        }

        unimplemented!()
    }
}

impl Queue for AutoQueue {
    fn head(&self) -> Option<usize> {
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
