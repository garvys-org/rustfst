use std::fmt::Display;

use crate::fst_traits::ExpandedFst;
use crate::semirings::WeightQuantize;
use crate::Semiring;

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
    let message = format!("Test {} with openfst failing : \nREF = \n{}\nPRED = \n{}\n \nREF = \n{:?}\nPRED = \n{:?}\n",
                          s.into(), fst_ref, fst_pred, fst_ref, fst_pred);
    assert!(fst_ref.equal_quantized(fst_pred), message)
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
