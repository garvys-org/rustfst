use fst::MutableFst;
use fst::ExpandedFst;
use semirings::Semiring;

pub fn compose<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> F3
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    F3::new()
}
