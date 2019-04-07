#[cfg(test)]
use crate::fst_impls::VectorFst;
#[cfg(test)]
use crate::fst_traits::CoreFst;
#[cfg(test)]
use crate::fst_traits::MutableFst;
#[cfg(test)]
use crate::semirings::IntegerWeight;
#[cfg(test)]
use crate::test_data::TestFst;
#[cfg(test)]
use crate::test_data::TestFstData;

#[cfg(test)]
pub(crate) struct EmptyFst {}
gen_test_fst!(EmptyFst);

#[cfg(test)]
impl TestFst for EmptyFst {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        Self::F::new()
    }

    fn get_name() -> String {
        String::from("vector_fst_001_empty")
    }

    fn get_connected_fst() -> Self::F {
        Self::F::new()
    }

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        vec![]
    }
}
