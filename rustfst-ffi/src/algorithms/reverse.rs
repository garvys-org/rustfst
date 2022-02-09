use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::RawPointerConverter;
use rustfst::algorithms::reverse;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[no_mangle]
pub extern "C" fn fst_union(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let res_fst: VectorFst<TropicalWeight> = reverse(vec_fst)?;
        unsafe { *ptr = CFst(Box::new(res_fst)).into_raw_pointer() };
        Ok(())
    })
}
