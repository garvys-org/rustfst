use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::{minimize_with_config, MinimizeConfig};
use rustfst::fst_impls::VectorFst;
use rustfst::prelude::minimize;
use rustfst::semirings::TropicalWeight;

#[derive(AsRust, CReprOf, CDrop, RawPointerConverter)]
#[target_type(MinimizeConfig)]
pub struct CMinimizeConfig {
    delta: f32,
    allow_nondet: bool,
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_minimize_config_new(
    delta: libc::c_float,
    allow_nondet: bool,
    ptr: *mut *const CMinimizeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let config = CMinimizeConfig {
            delta,
            allow_nondet,
        };
        unsafe { *ptr = config.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_minimize(ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        minimize(vec_fst)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_minimize_with_config(
    ptr: *mut CFst,
    config: *const CMinimizeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let config = unsafe {
            <CMinimizeConfig as ffi_convert::RawBorrow<CMinimizeConfig>>::raw_borrow(config)?
        };
        minimize_with_config(vec_fst, config.as_rust()?)?;
        Ok(())
    })
}
