#[cfg(test)]
use arc::Arc;
#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use fst_traits::MutableFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use test_data::TestFst;
#[cfg(test)]
use test_data::TestFstData;

#[cfg(test)]
pub(crate) struct VectorFst009 {}
gen_test_fst!(VectorFst009);

#[cfg(test)]
impl TestFst for VectorFst009 {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1).unwrap();
        fst.add_arc(&s1, Arc::new(3, 5, IntegerWeight::new(10), s2))
            .unwrap();
        fst.add_arc(&s1, Arc::new(5, 7, IntegerWeight::new(18), s2))
            .unwrap();
        fst.set_final(&s2, IntegerWeight::new(31)).unwrap();
        fst.add_state();
        let s4 = fst.add_state();
        fst.add_arc(&s2, Arc::new(5, 7, IntegerWeight::new(18), s4))
            .unwrap();
        fst
    }

    fn get_name() -> String {
        String::from("vector_fst_009")
    }

    fn get_connected_fst() -> Self::F {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        fst.set_start(&s1).unwrap();
        fst.add_arc(&s1, Arc::new(3, 5, IntegerWeight::new(10), s2))
            .unwrap();
        fst.add_arc(&s1, Arc::new(5, 7, IntegerWeight::new(18), s2))
            .unwrap();
        fst.set_final(&s2, IntegerWeight::new(31)).unwrap();
        fst
    }
}
