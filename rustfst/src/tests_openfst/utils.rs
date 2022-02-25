use std::fmt::Display;

use crate::algorithms::isomorphic;
use crate::fst_properties::FstProperties;
use crate::fst_traits::ExpandedFst;
use crate::semirings::WeightQuantize;
use crate::Semiring;
use crate::KDELTA;

pub fn test_correctness_properties<W: Semiring, FREF: ExpandedFst<W>, FPRED: ExpandedFst<W>>(
    fst_ref: &FREF,
    fst_pred: &FPRED,
    msg: String,
) {
    // The field FstProperties is correct as long as it doesn't contain properties that are not verified.
    // As such, doing an assert is not enough. I propose to check 2 things :
    // 1) Each property bit set in FREF is also set in FPRED
    // 2) Check that all the properties that are marked as verified in FPRED are effectively true.

    let props_fref = fst_ref.properties();
    let props_fpred = fst_pred.properties();
    assert!(
        props_fpred.contains(props_fref),
        "{} \n Props_fref = {:?}\nProps_fpred = {:?}",
        msg,
        props_fref,
        props_fpred
    );

    let mut known = FstProperties::empty();
    let computed_props_fpred = crate::fst_properties::compute_fst_properties(
        fst_pred,
        FstProperties::all_properties(),
        &mut known,
        false,
    )
    .unwrap();

    assert!(
        computed_props_fpred.contains(props_fpred),
        "{} \nComputed props = {:?}\nProps = {:?}",
        msg,
        computed_props_fpred,
        props_fpred
    );
}

pub fn test_num_epsilons<W: Semiring, FREF: ExpandedFst<W>, FPRED: ExpandedFst<W>>(
    fst_ref: &FREF,
    fst_pred: &FPRED,
    msg: String,
) {
    assert_eq!(fst_ref.num_states(), fst_pred.num_states(), "{}", &msg);
    for s in fst_ref.states_range() {
        assert_eq!(
            fst_ref.num_input_epsilons(s).unwrap(),
            fst_pred.num_input_epsilons(s).unwrap(),
            "{}",
            &msg
        );
        assert_eq!(
            fst_ref.num_output_epsilons(s).unwrap(),
            fst_pred.num_output_epsilons(s).unwrap(),
            "{}",
            &msg
        );
    }
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
    assert!(fst_ref.approx_equal(fst_pred, KDELTA), "{}", message);
    test_num_epsilons(fst_ref, fst_pred, message);
    test_correctness_properties(
        fst_ref,
        fst_pred,
        format!("Test properties {} with openfst failing", s),
    )
}

pub fn test_isomorphic_fst<
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
    assert!(isomorphic(fst_ref, fst_pred).unwrap(), "{}", message);
    test_num_epsilons(fst_ref, fst_pred, message);
    test_correctness_properties(
        fst_ref,
        fst_pred,
        format!("Test properties {} with openfst failing", s),
    )
}
