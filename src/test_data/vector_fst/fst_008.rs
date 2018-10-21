#[cfg(test)]
use arc::Arc;
#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use fst_traits::MutableFst;
#[cfg(test)]
use fst_traits::{CoreFst, ExpandedFst};
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use semirings::Semiring;
#[cfg(test)]
use test_data::TestFst;
#[cfg(test)]
use test_data::TestFstData;

#[cfg(test)]
pub(crate) struct VectorFst008 {}
gen_test_fst!(VectorFst008);

#[cfg(test)]
impl TestFst for VectorFst008 {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let mut fst = VectorFst::new();
        let s0 = fst.add_state();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s0).unwrap();
        fst.add_arc(&s0, Arc::new(3, 5, IntegerWeight::new(10), s1))
            .unwrap();
        fst.add_arc(&s0, Arc::new(5, 7, IntegerWeight::new(18), s1))
            .unwrap();
        fst.add_arc(&s1, Arc::new(3, 5, IntegerWeight::new(3), s2))
            .unwrap();
        fst.add_arc(&s1, Arc::new(5, 7, IntegerWeight::new(5), s2))
            .unwrap();
        fst.set_final(&s2, IntegerWeight::new(31)).unwrap();
        fst
    }

    fn get_name() -> String {
        String::from("vector_fst_008")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        let fst = Self::get_fst();
        let mut d = vec![vec![IntegerWeight::zero(); fst.num_states()]; fst.num_states()];

        d[0][0] = IntegerWeight::one();
        d[0][1] = IntegerWeight::new(10 + 18);
        d[0][2] = IntegerWeight::new((10 + 18) * (3 + 5));

        d[1][0] = IntegerWeight::zero();
        d[1][1] = IntegerWeight::one();
        d[1][2] = IntegerWeight::new(3 + 5);

        d[2][0] = IntegerWeight::zero();
        d[2][1] = IntegerWeight::zero();
        d[2][2] = IntegerWeight::one();

        d
    }
}
