use anyhow::Result;

use crate::algorithms::dfs_visit::{dfs_visit, Visitor};
use crate::algorithms::state_sort;
use crate::algorithms::tr_filters::AnyTrFilter;
use crate::fst_properties::FstProperties;
use crate::fst_traits::{Fst, MutableFst};
use crate::semirings::Semiring;
use crate::StateId;
use crate::Tr;

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

impl<'a, W: Semiring, F: 'a + Fst<W>> Visitor<'a, W, F> for TopOrderVisitor {
    fn init_visit(&mut self, _fst: &'a F) {}

    fn init_state(&mut self, _s: StateId, _root: StateId) -> bool {
        true
    }

    fn tree_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        true
    }

    fn back_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        self.acyclic = false;
        false
    }

    fn forward_or_cross_tr(&mut self, _s: StateId, _tr: &Tr<W>) -> bool {
        true
    }

    fn finish_state(&mut self, s: StateId, _parent: Option<StateId>, _tr: Option<&Tr<W>>) {
        self.finish.push(s)
    }

    fn finish_visit(&mut self) {
        if self.acyclic {
            self.order = vec![0; self.finish.len()];

            for s in 0..self.finish.len() {
                self.order[self.finish[self.finish.len() - s - 1] as usize] = s as StateId;
            }
        }
    }
}

/// Topologically sort an FST. When sorted, all transitions are from
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
pub fn top_sort<W, F>(fst: &mut F) -> Result<()>
where
    W: Semiring,
    F: MutableFst<W>,
{
    let mut visitor = TopOrderVisitor::new();
    dfs_visit(fst, &mut visitor, &AnyTrFilter {}, false);
    if visitor.acyclic {
        state_sort(fst, &visitor.order)?;
        let props =
            FstProperties::ACYCLIC | FstProperties::INITIAL_ACYCLIC | FstProperties::TOP_SORTED;
        fst.set_properties_with_mask(props, props);
    } else {
        let props = FstProperties::CYCLIC | FstProperties::NOT_TOP_SORTED;
        fst.set_properties_with_mask(props, props);
    }

    Ok(())
}
