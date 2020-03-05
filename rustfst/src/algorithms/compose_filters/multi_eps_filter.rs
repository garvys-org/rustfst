use failure::Fallible;

use crate::algorithms::compose_filters::ComposeFilter;
use crate::fst_traits::{CoreFst, Fst};
use crate::{Arc, NO_LABEL};

pub struct MultiEpsFilter<F> {
    filter: F,
    keep_multi_eps: bool,
}

impl<
        'matcher,
        'fst: 'matcher,
        F1: Fst + 'fst,
        F2: Fst<W = F1::W> + 'fst,
        F: ComposeFilter<'matcher, 'fst, F1, F2>,
    > ComposeFilter<'matcher, 'fst, F1, F2> for MultiEpsFilter<F>
{
    type M1 = F::M1;
    type M2 = F::M2;
    type FS = F::FS;

    fn new(fst1: &'fst F1, fst2: &'fst F2) -> Fallible<Self> {
        Ok(Self {
            filter: F::new(fst1, fst2)?,
            keep_multi_eps: false,
        })
    }

    fn start(&self) -> Self::FS {
        self.filter.start()
    }

    fn set_state(&mut self, s1: usize, s2: usize, filter_state: &Self::FS) {
        self.filter.set_state(s1, s2, filter_state)
    }

    fn filter_arc(
        &self,
        arc1: &mut Arc<<F1 as CoreFst>::W>,
        arc2: &mut Arc<<F2 as CoreFst>::W>,
    ) -> Option<Self::FS> {
        let opt_fs = self.filter.filter_arc(arc1, arc2);
        if self.keep_multi_eps {
            if arc1.olabel == NO_LABEL {
                arc1.ilabel = arc2.ilabel;
            }

            if arc2.ilabel == NO_LABEL {
                arc2.olabel = arc1.olabel;
            }
        }
        opt_fs
    }

    fn filter_final(&self, w1: &mut <F1 as CoreFst>::W, w2: &mut <F2 as CoreFst>::W) {
        self.filter.filter_final(w1, w2)
    }

    fn matcher1(&mut self) -> &'matcher mut Self::M1 {
        self.filter.matcher1()
    }

    fn matcher2(&mut self) -> &'matcher mut Self::M2 {
        self.filter.matcher2()
    }
}
