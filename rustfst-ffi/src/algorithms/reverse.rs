use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::RawPointerConverter;
use rustfst::algorithms::reverse;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_reverse(
    ptr: *const CFst,
    res_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let res_fst: VectorFst<TropicalWeight> = reverse(vec_fst)?;
        unsafe { *res_ptr = CFst(Box::new(res_fst)).into_raw_pointer() };
        Ok(())
    })
}
