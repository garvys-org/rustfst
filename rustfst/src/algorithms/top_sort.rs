use failure::Fallible;

use crate::algorithms::dfs_visit::{dfs_visit, Visitor};
use crate::algorithms::state_sort;
use crate::fst_traits::{ExpandedFst, Fst, MutableFst};
use crate::Arc;
use crate::StateId;

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
    fn init_visit(&mut self, _fst: &'a F) {}

    fn init_state(&mut self, _s: usize, _root: usize) -> bool {
        true
    }

    fn tree_arc(&mut self, _s: StateId, _arc: &Arc<F::W>) -> bool {
        true
    }

    fn back_arc(&mut self, _s: StateId, _arc: &Arc<F::W>) -> bool {
        self.acyclic = false;
        false
    }

    fn forward_or_cross_arc(&mut self, _s: StateId, _arc: &Arc<F::W>) -> bool {
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

/// This operation topologically sorts its input. When sorted, all transitions are from
/// lower to higher state IDs.
///
/// # Example
///
/// ## Input
///
/// ![topsort_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_in.svg?sanitize=true)
///
/// ## Output
///
/// ![topsort_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_out.svg?sanitize=true)
///
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
