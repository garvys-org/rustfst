#[cfg(test)]
use crate::arc::Arc;
#[cfg(test)]
use crate::fst_impls::VectorFst;
#[cfg(test)]
use crate::fst_traits::MutableFst;
#[cfg(test)]
use crate::fst_traits::{CoreFst, ExpandedFst};
#[cfg(test)]
use crate::semirings::IntegerWeight;
#[cfg(test)]
use crate::semirings::Semiring;
#[cfg(test)]
use crate::test_data::{TestFst, TestFstData};

#[cfg(test)]
pub(crate) struct VectorFst010 {}
gen_test_fst!(VectorFst010);

#[cfg(test)]
impl TestFst for VectorFst010 {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let mut fst = VectorFst::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(s0).unwrap();

        fst.add_arc(s0, Arc::new(3, 5, IntegerWeight::new(10), s1))
            .unwrap();
        fst.add_arc(s0, Arc::new(5, 7, IntegerWeight::new(18), s2))
            .unwrap();
        fst.set_final(s1, IntegerWeight::new(31)).unwrap();
        fst.set_final(s2, IntegerWeight::new(45)).unwrap();
        fst
    }

    fn get_name() -> String {
        String::from("vector_fst_010")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        let fst = Self::get_fst();
        let mut d = vec![vec![IntegerWeight::zero(); fst.num_states()]; fst.num_states()];

        d[0][0] = IntegerWeight::one();
        d[0][1] = IntegerWeight::new(10);
        d[0][2] = IntegerWeight::new(18);

        d[1][0] = IntegerWeight::zero();
        d[1][1] = IntegerWeight::one();
        d[1][2] = IntegerWeight::zero();

        d[2][0] = IntegerWeight::zero();
        d[2][1] = IntegerWeight::zero();
        d[2][2] = IntegerWeight::one();

        d
    }
}
