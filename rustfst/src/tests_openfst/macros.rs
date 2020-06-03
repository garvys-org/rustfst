use std::fmt::Display;

use crate::fst_traits::ExpandedFst;
use crate::semirings::WeightQuantize;
use crate::Semiring;

fn test_correctness_properties<W: Semiring, FREF: ExpandedFst<W>, FPRED: ExpandedFst<W>>(
    fst_ref: &FREF,
    fst_pred: &FPRED,
    msg: String,
) {
    // The field FstProperties is correct as long as it doesn't contain properties that are not verified.
    // As such, doing an assert is not enough. I propose to check 2 things :
    // 1) Each property bit set in FREF is also set in FPRED
    // 2) Check that all the properties that are marked as verified in FPRED are effectively true.

    let props_fref = fst_ref.properties_revamp();
    let props_fpred = fst_pred.properties_revamp();
    assert!(props_fpred.contains(props_fref));

    let computed_props_fpred = fst_pred.properties().unwrap();
    assert!(
        computed_props_fpred.contains(props_fpred),
        "{} \nComputed props = {:?}\nProps = {:?}",
        msg,
        computed_props_fpred,
        props_fpred
    );
}

pub fn test_eq_fst<
    W: Semiring + WeightQuantize,
    FREF: ExpandedFst<W> + Display,
    FPRED: ExpandedFst<W> + Display,
    I: Into<String>,
>(
    fst_ref: &FREF,
    fst_pred: &FPRED,
    s: I,
) {
    let s = s.into();
    let message = format!("Test {} with openfst failing : \nREF = \n{}\nPRED = \n{}\n \nREF = \n{:?}\nPRED = \n{:?}\n",
                          s, fst_ref, fst_pred, fst_ref, fst_pred);
    assert!(fst_ref.equal_quantized(fst_pred), message);
    test_correctness_properties(
        fst_ref,
        fst_pred,
        format!("Test properties {} with openfst failing", s),
    )
}

macro_rules! error_message_fst {
    ($fst_ref:expr, $fst:expr, $operation_name:expr) => {
        format!(
            "\nTest {} with openfst failing : \nREF = \n{}\nPRED = \n{}\n",
            $operation_name, $fst_ref, $fst
        )
    };
}

macro_rules! assert_eq_fst {
    ($fst_ref: expr, $fst: expr, $operation_name: expr) => {
        assert_eq!(
            $fst_ref,
            $fst,
            "{}",
            error_message_fst!($fst_ref, $fst, $operation_name)
        );
    };
}
