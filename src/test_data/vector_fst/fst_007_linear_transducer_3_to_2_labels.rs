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
pub(crate) struct LinearTransducer3to2Labels {}
gen_test_fst!(LinearTransducer3to2Labels);

#[cfg(test)]
impl TestFst for LinearTransducer3to2Labels {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let ilabels = vec![45, 58, 31];
        let olabels = vec![21, 18];
        transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_007_linear_transducer_3_to_2_labels")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }
}
