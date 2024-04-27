use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use rustfst::algorithms::isomorphic;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_isomorphic(
    fst: *const CFst,
    other_fst: *const CFst,
    is_isomorphic: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let other_fst = get!(CFst, other_fst);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let other_vec_fst: &VectorFst<TropicalWeight> = other_fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let res = isomorphic(vec_fst, other_vec_fst)?;
        unsafe { *is_isomorphic = res as usize }
        Ok(())
    })
}
