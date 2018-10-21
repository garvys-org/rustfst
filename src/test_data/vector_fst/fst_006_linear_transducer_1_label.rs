#[cfg(test)]
use fst_impls::VectorFst;
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
use utils::transducer;

#[cfg(test)]
pub(crate) struct LinearTransducerOneLabel {}
gen_test_fst!(LinearTransducerOneLabel);

#[cfg(test)]
impl TestFst for LinearTransducerOneLabel {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let ilabels = vec![32];
        let olabels = vec![45];
        transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_006_linear_transducer_one_label")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        let fst = Self::get_fst();
        let mut d = vec![vec![IntegerWeight::zero(); fst.num_states()]; fst.num_states()];

        d[0][0] = IntegerWeight::one();
        d[0][1] = IntegerWeight::one();

        d[1][0] = IntegerWeight::zero();
        d[1][1] = IntegerWeight::one();

        d
    }
}
