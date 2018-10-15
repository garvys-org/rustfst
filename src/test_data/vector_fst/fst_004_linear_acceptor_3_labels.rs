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
pub(crate) struct LinearAcceptor3Labels {}
gen_test_fst!(LinearAcceptor3Labels);

#[cfg(test)]
impl TestFst for LinearAcceptor3Labels {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let labels = vec![45, 58, 31];
        acceptor(labels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("vector_fst_004_linear_acceptor_three_labels")
    }

    fn get_connected_fst() -> Self::F {
        Self::get_fst()
    }
}
