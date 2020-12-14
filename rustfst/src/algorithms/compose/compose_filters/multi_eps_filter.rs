use std::borrow::Borrow;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::sync::Arc;

use anyhow::Result;

use crate::algorithms::compose::compose_filters::ComposeFilter;
use crate::algorithms::compose::matchers::Matcher;
use crate::fst_properties::FstProperties;
use crate::fst_traits::Fst;
use crate::semirings::Semiring;
use crate::{StateId, Tr, NO_LABEL};

#[derive(Debug, Clone)]
pub struct MultiEpsFilter<W, F1, F2, B1, B2, M1, M2, CF>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CF: ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    filter: CF,
    keep_multi_eps: bool,
    ghost: PhantomData<(W, F1, F2, B1, B2, M1, M2)>,
}

impl<W, F1, F2, B1, B2, M1, M2, CF: ComposeFilter<W, F1, F2, B1, B2, M1, M2>>
    ComposeFilter<W, F1, F2, B1, B2, M1, M2> for MultiEpsFilter<W, F1, F2, B1, B2, M1, M2, CF>
where
    W: Semiring,
    F1: Fst<W>,
    F2: Fst<W>,
    B1: Borrow<F1> + Debug,
    B2: Borrow<F2> + Debug,
    M1: Matcher<W, F1, B1>,
    M2: Matcher<W, F2, B2>,
    CF: ComposeFilter<W, F1, F2, B1, B2, M1, M2>,
{
    type FS = CF::FS;

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: StateId, s2: StateId, filter_state: &Self::FS) -> Result<()> {
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

    fn matcher1(&self) -> &M1 {
        self.filter.matcher1()
    }

    fn matcher2(&self) -> &M2 {
        self.filter.matcher2()
    }

    fn matcher1_shared(&self) -> &Arc<M1> {
        self.filter.matcher1_shared()
    }

    fn matcher2_shared(&self) -> &Arc<M2> {
        self.filter.matcher2_shared()
    }

    fn properties(&self, inprops: FstProperties) -> FstProperties {
        let oprops = self.filter.properties(inprops);
        oprops
            & FstProperties::i_label_invariant_properties()
            & FstProperties::o_label_invariant_properties()
    }
}
