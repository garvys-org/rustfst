use fst::MutableFst;

pub fn union<W, F1, F2, F3>(fst_1: &F1, fst_2: &F2) -> F3
where
    W: Semiring,
    F1: ExpandedFst<W = W>,
    F2: ExpandedFst<W = W>,
    F3: MutableFst<W = W>,
{
    let mut fst_out = F3::new();

    fst_out
}
