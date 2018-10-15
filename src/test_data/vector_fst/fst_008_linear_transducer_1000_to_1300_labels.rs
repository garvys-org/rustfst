#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use test_data::TestFst;
#[cfg(test)]
use test_data::TestFstData;
#[cfg(test)]
use utils::transducer;

#[cfg(test)]
pub(crate) struct LinearTransducer1000to1300Labels {}
gen_test_fst!(LinearTransducer1000to1300Labels);

#[cfg(test)]
impl TestFst for LinearTransducer1000to1300Labels {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        transducer(0..1000, 0..1300).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_008_linear_transducer_1000_to_1300_labels")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }
}
