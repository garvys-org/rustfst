use std::borrow::Borrow;

use anyhow::Result;

use crate::algorithms::lazy_fst_revamp::{LazyFst, SimpleHashMapCache};
use crate::algorithms::replace_mod::config::ReplaceFstOptions;
use crate::algorithms::replace_mod::replace_fst_impl::ReplaceFstImpl;
use crate::fst_traits::Fst;
use crate::Label;
use crate::semirings::Semiring;

/// ReplaceFst supports lazy replacement of trs in one FST with another FST.
/// This replacement is recursive. ReplaceFst can be used to support a variety of
/// delayed constructions such as recursive transition networks, union, or closure.
pub type ReplaceFst<W, F, B> = LazyFst<W, ReplaceFstImpl<W, F, B>, SimpleHashMapCache<W>>;

impl<W: Semiring, F: Fst<W>, B: Borrow<F>> ReplaceFst<W, F, B>
{
    pub fn new(fst_list: Vec<(Label, B)>, root: Label, epsilon_on_replace: bool) -> Result<Self> {
        let mut isymt = None;
        let mut osymt = None;
        if let Some(first_elt) = fst_list.first() {
            isymt = first_elt.1.borrow().input_symbols().cloned();
            osymt = first_elt.1.borrow().output_symbols().cloned();
        }
        let opts = ReplaceFstOptions::new(root, epsilon_on_replace);
        let fst_op = ReplaceFstImpl::new(fst_list, opts)?;
        let fst_cache = SimpleHashMapCache::new();
        Ok(Self::from_op_and_cache(fst_op, fst_cache, isymt, osymt))
    }
}
