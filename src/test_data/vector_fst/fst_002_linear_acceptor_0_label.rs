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
use utils::acceptor;

#[cfg(test)]
pub(crate) struct LinearAcceptor0Label {}
gen_test_fst!(LinearAcceptor0Label);

#[cfg(test)]
impl TestFst for LinearAcceptor0Label {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let labels = vec![];
        acceptor(labels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_002_linear_acceptor_zero_label")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }

    fn get_all_distances() -> Vec<Vec<<<Self as TestFst>::F as CoreFst>::W>> {
        let fst = Self::get_fst();
        let mut d = vec![vec![IntegerWeight::zero(); fst.num_states()]; fst.num_states()];

        d[0][0] = IntegerWeight::one();

        d
    }
}
