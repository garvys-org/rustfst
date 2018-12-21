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
use crate::utils::acceptor;

#[cfg(test)]
pub(crate) struct LinearAcceptor1Label {}
gen_test_fst!(LinearAcceptor1Label);

#[cfg(test)]
impl TestFst for LinearAcceptor1Label {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let labels = vec![32];
        acceptor(labels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_003_linear_acceptor_one_label")
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
