use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::semirings::Semiring;
use crate::{Tr, NO_LABEL};

#[derive(Debug, Clone)]
pub struct MultiEpsFilter<F> {
    filter: F,
    keep_multi_eps: bool,
}

impl<W: Semiring, F: ComposeFilter<W>> ComposeFilter<W> for MultiEpsFilter<F> {
    type M1 = F::M1;
    type M2 = F::M2;
    type FS = F::FS;

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) -> Result<()> {
        self.filter.set_state(s1, s2, filter_state)
    }

    fn filter_tr(&mut self, arc1: &mut Tr<W>, arc2: &mut Tr<W>) -> Result<Self::FS> {
        let opt_fs = self.filter.filter_tr(arc1, arc2)?;
        if self.keep_multi_eps {
            if arc1.olabel == NO_LABEL {
                arc1.ilabel = arc2.ilabel;
            }

            if arc2.ilabel == NO_LABEL {
                arc2.olabel = arc1.olabel;
            }
        }
        Ok(opt_fs)
    }

    fn filter_final(&self, w1: &mut W, w2: &mut W) -> Result<()> {
        self.filter.filter_final(w1, w2)
    }

    fn matcher1(&self) -> &Self::M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &Self::M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<Self::M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<Self::M2> {
        self.filter.matcher2_shared()
    }
}
