#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use test_data::TestFst;
#[cfg(test)]
use test_data::TestFstData;
#[cfg(test)]
use utils::acceptor;

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
}
