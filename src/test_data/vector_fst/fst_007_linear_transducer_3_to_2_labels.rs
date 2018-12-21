#[cfg(test)]
use crate::fst_impls::VectorFst;
#[cfg(test)]
use crate::fst_traits::{CoreFst, ExpandedFst};
#[cfg(test)]
use crate::semirings::IntegerWeight;
#[cfg(test)]
use crate::semirings::Semiring;
#[cfg(test)]
use crate::test_data::TestFst;
#[cfg(test)]
use crate::test_data::TestFstData;
#[cfg(test)]
use crate::utils::transducer;

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

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        let fst = Self::get_fst();
        let mut d = vec![vec![IntegerWeight::zero(); fst.num_states()]; fst.num_states()];

        d[0][0] = IntegerWeight::one();
        d[0][1] = IntegerWeight::one();
        d[0][2] = IntegerWeight::one();
        d[0][3] = IntegerWeight::one();

        d[1][0] = IntegerWeight::zero();
        d[1][1] = IntegerWeight::one();
        d[1][2] = IntegerWeight::one();
        d[1][3] = IntegerWeight::one();

        d[2][0] = IntegerWeight::zero();
        d[2][1] = IntegerWeight::zero();
        d[2][2] = IntegerWeight::one();
        d[2][3] = IntegerWeight::one();

        d[3][0] = IntegerWeight::zero();
        d[3][1] = IntegerWeight::zero();
        d[3][2] = IntegerWeight::zero();
        d[3][3] = IntegerWeight::one();

        d
    }
}
