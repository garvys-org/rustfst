use crate::algorithms::dfs_visit::dfs_visit;
use crate::algorithms::top_sort::TopOrderVisitor;
use crate::algorithms::tr_filters::TrFilter;
use crate::algorithms::{Queue, QueueType};
use crate::fst_traits::ExpandedFst;
use crate::semirings::Semiring;
use crate::StateId;

/// Topological-order queue discipline, templated on the StateId. States are
/// ordered in the queue topologically. The FST must be acyclic.
#[derive(Debug, Clone)]
pub struct TopOrderQueue {
    order: Vec<StateId>,
    state: Vec<Option<StateId>>,
    front: StateId,
    back: Option<StateId>,
}

impl TopOrderQueue {
    pub fn new<W: Semiring, F: ExpandedFst<W>, A: TrFilter<W>>(fst: &F, tr_filter: &A) -> Self {
        let mut visitor = TopOrderVisitor::new();
        dfs_visit(fst, &mut visitor, tr_filter, false);
        if !visitor.acyclic {
            panic!("Unexpectted Acyclic FST for TopOprerQueue");
        }
        Self::from_precomputed_order(visitor.order)
    }

    pub fn from_precomputed_order(order: Vec<StateId>) -> Self {
        let order_len = order.len();
        Self {
            order,
            state: vec![None; order_len],
            front: 0,
            back: None,
        }
    }
}

impl Queue for TopOrderQueue {
    fn head(&mut self) -> Option<StateId> {
        self.state[self.front as usize]
    }

    fn enqueue(&mut self, state: StateId) {
        let u_state = state as usize;
        if self.back.is_none() || self.front > self.back.unwrap() {
            self.front = self.order[u_state];
            self.back = Some(self.order[u_state]);
        } else if self.order[u_state] > self.back.unwrap() {
            self.back = Some(self.order[u_state]);
        } else if self.order[u_state] < self.front {
            self.front = self.order[u_state];
        }
        self.state[self.order[u_state] as usize] = Some(state);
    }

    fn dequeue(&mut self) -> Option<StateId> {
        if self.is_empty() {
            return None;
        }
        let old_head = self.state[self.front as usize].take();
        if self.back.is_some() {
            while self.front <= self.back.unwrap() && self.state[self.front as usize].is_none() {
                self.front += 1;
            }
        }
        old_head
    }

    fn update(&mut self, _state: StateId) {}

    fn is_empty(&self) -> bool {
        if let Some(back_) = self.back {
            self.front > back_
        } else {
            true
        }
    }

    fn clear(&mut self) {
        if let Some(back_) = self.back {
            for s in self.front..=back_ {
                self.state[s as usize] = None;
            }
        }
        self.front = 0;
        self.back = None;
    }

    fn queue_type(&self) -> QueueType {
        QueueType::TopOrderQueue
    }
}
