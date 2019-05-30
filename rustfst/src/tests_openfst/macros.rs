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
