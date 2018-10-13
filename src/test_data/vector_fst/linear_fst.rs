use fst_impls::VectorFst;
use semirings::IntegerWeight;
use test_data::TestFst;
use utils::{acceptor, transducer};

pub fn get_linear_fsts() -> Vec<VectorFst<IntegerWeight>> {
    let mut vec = vec![];
    //    vec.push(linear_acceptor_empty());

    vec
}

pub struct LinearAcceptorEmpty {}

impl TestFst for LinearAcceptorEmpty {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let labels = vec![];
        acceptor(labels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("linear_acceptor_empty")
    }
}

pub struct LinearAcceptorOneLabel {}
impl TestFst for LinearAcceptorOneLabel {
    type F = VectorFst<IntegerWeight>;

    fn get_fst() -> <Self as TestFst>::F {
        let labels = vec![32];
        acceptor(labels.into_iter()).unwrap()
    }

    fn get_name() -> String {
        String::from("linear_acceptor_one_label")
    }
}

pub fn linear_acceptor_3_labels() -> VectorFst<IntegerWeight> {
    let labels = vec![45, 58, 31];
    acceptor(labels.into_iter()).unwrap()
}

pub fn linear_acceptor_1000_labels() -> VectorFst<IntegerWeight> {
    acceptor(0..1000).unwrap()
}

pub fn linear_trasnducer_one_label() -> VectorFst<IntegerWeight> {
    let ilabels = vec![32];
    let olabels = vec![45];
    transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
}

pub fn linear_transducer_3_2_labels() -> VectorFst<IntegerWeight> {
    let ilabels = vec![45, 58, 31];
    let olabels = vec![21, 18];
    transducer(ilabels.into_iter(), olabels.into_iter()).unwrap()
}

pub fn linear_transducer_1000_1300_labels() -> VectorFst<IntegerWeight> {
    transducer(0..1000, 0..1300).unwrap()
}
