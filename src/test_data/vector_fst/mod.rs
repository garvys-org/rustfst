pub(crate) mod linear_fst;

#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use std::vec::IntoIter;
#[cfg(test)]
use test_data::TestFstData;

#[cfg(test)]
pub(crate) fn get_vector_fsts_for_tests() -> IntoIter<TestFstData<VectorFst<IntegerWeight>>> {
    let res = linear_fst::get_linear_fsts();
    res
}

// TODO : Add empty FST
// TODO : Add not connected FST
