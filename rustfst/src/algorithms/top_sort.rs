use std::collections::HashSet;

use failure::Fallible;

use crate::algorithms::state_sort;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::Arc;
use crate::StateId;
use crate::NO_STATE_ID;

use crate::algorithms::dfs_visit::{dfs_visit, Visitor};

pub struct TopOrderVisitor {
    pub order: Vec<StateId>,
    pub acyclic: bool,
    pub finish: Vec<StateId>,
}

impl TopOrderVisitor {
    pub fn new() -> Self {
        Self {
            order: vec![],
            acyclic: true,
            finish: vec![],
        }
    }
}

impl<'a, F: 'a + Fst> Visitor<'a, F> for TopOrderVisitor {
    fn init_visit(&mut self, fst: &'a F) {}

    fn init_state(&mut self, s: usize, root: usize) -> bool {
        true
    }

    fn tree_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool {
        true
    }

    fn back_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool {
        self.acyclic = false;
        false
    }

    fn forward_or_cross_arc(&mut self, s: StateId, arc: &Arc<F::W>) -> bool {
        true
    }

    fn finish_state(&mut self, s: StateId, _parent: Option<StateId>, _arc: Option<&Arc<F::W>>) {
        self.finish.push(s)
    }

    fn finish_visit(&mut self) {
        if self.acyclic {
            self.order = vec![0; self.finish.len()];

            for s in 0..self.finish.len() {
                self.order[self.finish[self.finish.len() - s - 1]] = s;
            }
        }
    }
}

pub fn top_sort<F>(fst: &mut F) -> Fallible<()>
where
    F: MutableFst + ExpandedFst,
{
    let mut visitor = TopOrderVisitor::new();
    dfs_visit(fst, &mut visitor, false);
    if visitor.acyclic {
        state_sort(fst, &visitor.order)?;
    }

    Ok(())
}
