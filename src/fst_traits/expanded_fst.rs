use fst_traits::Fst;

pub trait ExpandedFst: Fst {
    fn num_states(&self) -> usize;
}
