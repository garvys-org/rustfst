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
pub(crate) struct VectorFst010 {}
gen_test_fst!(VectorFst010);

#[cfg(test)]
impl TestFst for VectorFst010 {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let mut fst = VectorFst::new();
        let s1 = fst.add_state();
        let s2 = fst.add_state();
        let s3 = fst.add_state();
        fst.set_start(&s1).unwrap();

        fst.add_arc(&s1, Arc::new(3, 5, IntegerWeight::new(10), s2))
            .unwrap();
        fst.add_arc(&s1, Arc::new(5, 7, IntegerWeight::new(18), s3))
            .unwrap();
        fst.set_final(&s2, IntegerWeight::new(31)).unwrap();
        fst.set_final(&s3, IntegerWeight::new(45)).unwrap();
        fst
    }

    fn get_name() -> String {
        String::from("vector_fst_010")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }
}
