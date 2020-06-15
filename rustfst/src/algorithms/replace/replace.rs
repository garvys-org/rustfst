use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::replace::ReplaceFst;
use crate::fst_traits::{AllocableFst, Fst, MutableFst};
use crate::semirings::Semiring;
use crate::Label;

/// Recursively replaces trs in the root FSTs with other FSTs.
///
/// Replace supports replacement of trs in one Fst with another FST. This
/// replacement is recursive. Replace takes an array of FST(s). One FST
/// represents the root (or topology) machine. The root FST refers to other FSTs
/// by recursively replacing trs labeled as non-terminals with the matching
/// non-terminal FST. Currently Replace uses the output symbols of the trs to
/// determine whether the transition is a non-terminal transition or not. A non-terminal can be
/// any label that is not a non-zero terminal label in the output alphabet.
///
/// Note that input argument is a vector of pairs. These correspond to the tuple
/// of non-terminal Label and corresponding FST.
///
/// # Example
///
/// ## Root Fst
///
/// ![replace_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_1.svg?sanitize=true)
///
/// ## Fst for non-terminal #NAME
///
/// ![replace_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_2.svg?sanitize=true)
///
/// ## Fst for non-termincal #FIRSTNAME
///
/// ![replace_in_3](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_3.svg?sanitize=true)
///
/// ## Fst for non-termincal #LASTNAME
///
/// ![replace_in_4](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_4.svg?sanitize=true)
///
/// ## Output
///
/// ![replace_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_out.svg?sanitize=true)
///
pub fn replace<W, F1, F2, B>(
    fst_list: Vec<(Label, B)>,
    root: Label,
    epsilon_on_replace: bool,
) -> Result<F2>
where
    F1: Fst<W>,
    W: Semiring,
    F2: MutableFst<W> + AllocableFst<W>,
    B: Borrow<F1>,
{
    let fst = ReplaceFst::new(fst_list, root, epsilon_on_replace)?;
    fst.compute()
}
