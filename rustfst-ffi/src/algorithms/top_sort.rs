use anyhow::anyhow;

use crate::fst::as_mut_fst;
use crate::fst::CFst;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};

use rustfst::algorithms::top_sort;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_top_sort(ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        top_sort(vec_fst)?;
        Ok(())
    })
}
