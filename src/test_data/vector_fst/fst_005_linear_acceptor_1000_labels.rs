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
pub(crate) struct LinearAcceptor1000Labels {}
gen_test_fst!(LinearAcceptor1000Labels);

#[cfg(test)]
impl TestFst for LinearAcceptor1000Labels {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        acceptor(0..1000).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_005_linear_acceptor_1000_labels")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }
}
