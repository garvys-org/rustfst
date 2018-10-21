pub(crate) mod fst_001_empty;
pub(crate) mod fst_002_linear_acceptor_0_label;
pub(crate) mod fst_003_linear_acceptor_1_label;
pub(crate) mod fst_004_linear_acceptor_3_labels;
pub(crate) mod fst_006_linear_transducer_1_label;
pub(crate) mod fst_007_linear_transducer_3_to_2_labels;
pub(crate) mod fst_008;
pub(crate) mod fst_009;
pub(crate) mod fst_010;

#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use std::vec::IntoIter;
#[cfg(test)]
use test_data::TestFstData;

#[cfg(test)]
pub(crate) fn get_linear_fsts() -> Vec<TestFstData<VectorFst<IntegerWeight>>> {
    let mut vec = vec![];
    vec.push(fst_001_empty::EmptyFst::new().into());
    vec.push(fst_002_linear_acceptor_0_label::LinearAcceptor0Label::new().into());
    vec.push(fst_003_linear_acceptor_1_label::LinearAcceptor1Label::new().into());
    vec.push(fst_004_linear_acceptor_3_labels::LinearAcceptor3Labels::new().into());
    vec.push(fst_006_linear_transducer_1_label::LinearTransducerOneLabel::new().into());
    vec.push(fst_007_linear_transducer_3_to_2_labels::LinearTransducer3to2Labels::new().into());
    vec
}

#[cfg(test)]
pub(crate) fn get_vector_fsts_for_tests() -> IntoIter<TestFstData<VectorFst<IntegerWeight>>> {
    let mut res = get_linear_fsts();
    res.push(fst_008::VectorFst008::new().into());
    res.push(fst_009::VectorFst009::new().into());
    res.push(fst_010::VectorFst010::new().into());
    res.into_iter()
}

// TODO : Add not connected FST
