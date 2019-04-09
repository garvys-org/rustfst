use std::collections::HashSet;

use crate::StateId;
use crate::algorithms::{Queue, QueueType};
use crate::fst_traits::{MutableFst, ExpandedFst};
use crate::algorithms::top_sort::dfs_topsort;

/// Topological-order queue discipline, templated on the StateId. States are
/// ordered in the queue topologically. The FST must be acyclic.
pub struct TopOrderQueue {
    order: Vec<StateId>,
    state: Vec<Option<StateId>>
}

impl TopOrderQueue {
    pub fn new<F: MutableFst + ExpandedFst>(fst: &F) -> Self {
        let mut accessible_states = HashSet::new();
        let mut order = vec![];
        dfs_topsort(fst, &mut accessible_states, &mut order);
        let order_len = order.len();
        Self { order, state:  vec![None; order_len]}
    }
}

impl Queue for TopOrderQueue {
    fn head(&self) -> Option<usize> {
        unimplemented!()
    }

    fn enqueue(&mut self, state: usize) {
        unimplemented!()
    }

    fn dequeue(&mut self) {
        unimplemented!()
    }

    fn update(&mut self, state: usize) {
        unimplemented!()
    }

    fn is_empty(&self) -> bool {
        unimplemented!()
    }

    fn clear(&mut self) {
        unimplemented!()
    }

    fn queue_type() -> QueueType {
        QueueType::TopOrderQueue
    }
}
