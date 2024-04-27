use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, get_mut, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::RawPointerConverter;
use rustfst::algorithms::weight_converters::SimpleWeightConverter;
use rustfst::algorithms::{optimize, weight_convert};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::LogWeight;
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_optimize(ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        optimize(vec_fst)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_optimize_in_log(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };

        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let mut converter = SimpleWeightConverter {};
        let mut vec_log_fst: VectorFst<LogWeight> = weight_convert(vec_fst, &mut converter)?;
        optimize(&mut vec_log_fst)?;
        let res_fst: VectorFst<TropicalWeight> = weight_convert(&vec_log_fst, &mut converter)?;
        let res_ptr = CFst(Box::new(res_fst)).into_raw_pointer();
        unsafe { *ptr = res_ptr };
        Ok(())
    })
}
