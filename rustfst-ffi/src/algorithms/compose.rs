use anyhow::{anyhow, Result};

use super::EnumConversionError;
use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::compose::{
    compose, compose_with_config, ComposeConfig, ComposeFilterEnum,
};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[derive(RawPointerConverter, Debug)]
pub struct CComposeFilterEnum(pub(crate) usize);

impl AsRust<ComposeFilterEnum> for CComposeFilterEnum {
    fn as_rust(&self) -> Result<ComposeFilterEnum, AsRustError> {
        match self.0 {
            0 => Ok(ComposeFilterEnum::AutoFilter),
            1 => Ok(ComposeFilterEnum::NullFilter),
            2 => Ok(ComposeFilterEnum::TrivialFilter),
            3 => Ok(ComposeFilterEnum::SequenceFilter),
            4 => Ok(ComposeFilterEnum::AltSequenceFilter),
            5 => Ok(ComposeFilterEnum::MatchFilter),
            6 => Ok(ComposeFilterEnum::NoMatchFilter),
            _ => Err(AsRustError::Other(Box::new(EnumConversionError {}))),
        }
    }
}

impl CDrop for CComposeFilterEnum {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl CReprOf<ComposeFilterEnum> for CComposeFilterEnum {
    fn c_repr_of(value: ComposeFilterEnum) -> Result<CComposeFilterEnum, CReprOfError> {
        let variant = match value {
            ComposeFilterEnum::AutoFilter => 0,
            ComposeFilterEnum::NullFilter => 1,
            ComposeFilterEnum::TrivialFilter => 2,
            ComposeFilterEnum::SequenceFilter => 3,
            ComposeFilterEnum::AltSequenceFilter => 4,
            ComposeFilterEnum::MatchFilter => 5,
            ComposeFilterEnum::NoMatchFilter => 6,
        };
        Ok(CComposeFilterEnum(variant))
    }
}

#[derive(AsRust, CReprOf, CDrop, RawPointerConverter, Debug)]
#[target_type(ComposeConfig)]
pub struct CComposeConfig {
    pub compose_filter: CComposeFilterEnum,
    pub connect: bool,
}

#[no_mangle]
pub extern "C" fn fst_compose_config_new(
    compose_filter: libc::size_t,
    connect: bool,
    config: *mut *const CComposeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let compose_config = CComposeConfig {
            compose_filter: CComposeFilterEnum(compose_filter as usize),
            connect,
        };
        unsafe { *config = compose_config.into_raw_pointer() };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_compose(
    fst_1: *const CFst,
    fst_2: *const CFst,
    composition_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_1 = get!(CFst, fst_1);
        let vec_fst1: &VectorFst<TropicalWeight> = fst_1
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst_2 = get!(CFst, fst_2);
        let vec_fst2: &VectorFst<TropicalWeight> = fst_2
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst: VectorFst<TropicalWeight> = compose::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            _,
            _,
            _,
        >(vec_fst1, vec_fst2)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *composition_ptr = fst_ptr };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_compose_with_config(
    fst_1: *const CFst,
    fst_2: *const CFst,
    config: *const CComposeConfig,
    composition_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_1 = get!(CFst, fst_1);
        let vec_fst1: &VectorFst<TropicalWeight> = fst_1
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst_2 = get!(CFst, fst_2);
        let vec_fst2: &VectorFst<TropicalWeight> = fst_2
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let compose_config = unsafe {
            <CComposeConfig as ffi_convert::RawBorrow<CComposeConfig>>::raw_borrow(config)?
        };
        let fst: VectorFst<TropicalWeight> =
            compose_with_config::<
                TropicalWeight,
                VectorFst<TropicalWeight>,
                VectorFst<TropicalWeight>,
                _,
                _,
                _,
            >(vec_fst1, vec_fst2, compose_config.as_rust()?)?;
        let fst_ptr = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *composition_ptr = fst_ptr };
        Ok(())
    })
}
