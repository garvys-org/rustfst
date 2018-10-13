use fst_traits::Fst;

pub(crate) trait TestFst {
    type F: Fst;
    fn get_fst() -> Self::F;
    fn get_name() -> String;
}
