use anyhow::{anyhow, Result};

use super::EnumConversionError;
use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::determinize::{
    determinize, determinize_with_config, DeterminizeConfig, DeterminizeType,
};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[derive(RawPointerConverter)]
pub struct CDeterminizeType(usize);

impl AsRust<DeterminizeType> for CDeterminizeType {
    fn as_rust(&self) -> Result<DeterminizeType, AsRustError> {
        match self.0 {
            0 => Ok(DeterminizeType::DeterminizeFunctional),
            1 => Ok(DeterminizeType::DeterminizeNonFunctional),
            2 => Ok(DeterminizeType::DeterminizeDisambiguate),
            _ => Err(AsRustError::Other(Box::new(EnumConversionError {}))),
        }
    }
}

impl CDrop for CDeterminizeType {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl CReprOf<DeterminizeType> for CDeterminizeType {
    fn c_repr_of(value: DeterminizeType) -> Result<CDeterminizeType, CReprOfError> {
        let variant = match value {
            DeterminizeType::DeterminizeFunctional => 0,
            DeterminizeType::DeterminizeNonFunctional => 1,
            DeterminizeType::DeterminizeDisambiguate => 2,
        };
        Ok(CDeterminizeType(variant))
    }
}

#[derive(AsRust, CReprOf, CDrop, RawPointerConverter)]
#[target_type(DeterminizeConfig)]
pub struct CDeterminizeConfig {
    delta: f32,
    det_type: CDeterminizeType,
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_determinize_config_new(
    delta: libc::c_float,
    det_type: libc::size_t,
    config: *mut *const CDeterminizeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let determinize_config = CDeterminizeConfig {
            delta,
            det_type: CDeterminizeType(det_type),
        };
        unsafe { *config = determinize_config.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_determinize(
    ptr: *const CFst,
    det_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst: VectorFst<TropicalWeight> = determinize(vec_fst)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *det_fst = fst_ptr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_determinize_with_config(
    ptr: *const CFst,
    config: *const CDeterminizeConfig,
    det_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let det_config = unsafe {
            <CDeterminizeConfig as ffi_convert::RawBorrow<CDeterminizeConfig>>::raw_borrow(config)?
        };
        let fst: VectorFst<TropicalWeight> =
            determinize_with_config(vec_fst, det_config.as_rust()?)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *det_fst = fst_ptr };
        Ok(())
    })
}
