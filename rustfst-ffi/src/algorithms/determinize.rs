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

#[no_mangle]
pub extern "C" fn fst_determinize(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let fst: VectorFst<TropicalWeight> = determinize(vec_fst)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *ptr = fst_ptr };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_determinize_with_config(
    ptr: *mut *const CFst,
    config: *const CDeterminizeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let det_config = unsafe {
            <CDeterminizeConfig as ffi_convert::RawBorrow<CDeterminizeConfig>>::raw_borrow(config)?
        };

        let fst: VectorFst<TropicalWeight> =
            determinize_with_config(vec_fst, det_config.as_rust()?)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *ptr = fst_ptr };
        Ok(())
    })
}
