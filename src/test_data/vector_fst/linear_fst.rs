#[cfg(test)]
use fst_impls::VectorFst;
#[cfg(test)]
use semirings::IntegerWeight;
#[cfg(test)]
use std::vec::IntoIter;
#[cfg(test)]
use test_data::TestFst;
#[cfg(test)]
use test_data::TestFstData;
#[cfg(test)]
use utils::{acceptor, transducer};

#[cfg(test)]
pub(crate) fn get_linear_fsts() -> IntoIter<TestFstData<VectorFst<IntegerWeight>>> {
    let mut vec = vec![];
    vec.push(LinearAcceptor0Label::new().into());
    vec.push(LinearAcceptor1Label::new().into());
    vec.push(LinearAcceptor3Labels::new().into());
    vec.push(LinearAcceptor1000Labels::new().into());
    vec.push(LinearTransducerOneLabel::new().into());
    vec.push(LinearTransducer3to2Labels::new().into());
    vec.push(LinearTransducer1000to1300Labels::new().into());
    vec.into_iter()
}

#[cfg(test)]
pub(crate) struct LinearAcceptor0Label {}
gen_test_fst!(
    LinearAcceptor0Label,
    {
        let labels = vec![];
        acceptor(labels.into_iter()).unwrap()
    },
    "linear_acceptor_zero_label"
);

#[cfg(test)]
pub(crate) struct LinearAcceptor1Label {}
gen_test_fst!(
    LinearAcceptor1Label,
    {
        let labels = vec![32];
        acceptor(labels.into_iter()).unwrap()
    },
    "linear_acceptor_one_label"
);

#[cfg(test)]
pub(crate) struct LinearAcceptor3Labels {}
gen_test_fst!(
    LinearAcceptor3Labels,
    {
        let labels = vec![45, 58, 31];
        acceptor(labels.into_iter()).unwrap()
    },
    "linear_acceptor_three_labels"
);

#[cfg(test)]
pub(crate) struct LinearAcceptor1000Labels {}
gen_test_fst!(
    LinearAcceptor1000Labels,
    { acceptor(0..1000).unwrap() },
    "linear_acceptor_1000_labels"
);

#[cfg(test)]
pub(crate) struct LinearTransducerOneLabel {}
gen_test_fst!(
    LinearTransducerOneLabel,
    {
        let ilabels = vec![32];
        let olabels = vec![45];
        transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
    },
    "linear_transducer_one_label"
);

#[cfg(test)]
pub(crate) struct LinearTransducer3to2Labels {}
gen_test_fst!(
    LinearTransducer3to2Labels,
    {
        let ilabels = vec![45, 58, 31];
        let olabels = vec![21, 18];
        transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
    },
    "linear_transducer_3_to_2_labels"
);

#[cfg(test)]
pub(crate) struct LinearTransducer1000to1300Labels {}
gen_test_fst!(
    LinearTransducer1000to1300Labels,
    { transducer(0..1000, 0..1300).unwrap() },
    "linear_transducer_1000_to_1300_labels"
);
