use anyhow::{anyhow, Result};

use super::EnumConversionError;
use crate::fst::CFst;
use crate::{get, wrap, CLabel, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::compose::matchers::MatcherRewriteMode;
use rustfst::algorithms::compose::{
    compose, compose_with_config, ComposeConfig, ComposeFilterEnum, MatcherConfig,
    SigmaMatcherConfig,
};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;
use rustfst::Label;

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

#[derive(RawPointerConverter, Debug, Clone)]
pub struct CMatcherRewriteMode(pub(crate) usize);

impl AsRust<MatcherRewriteMode> for CMatcherRewriteMode {
    fn as_rust(&self) -> Result<MatcherRewriteMode, AsRustError> {
        match self.0 {
            0 => Ok(MatcherRewriteMode::MatcherRewriteAuto),
            1 => Ok(MatcherRewriteMode::MatcherRewriteAlways),
            2 => Ok(MatcherRewriteMode::MatcherRewriteNever),
            _ => Err(AsRustError::Other(Box::new(EnumConversionError {}))),
        }
    }
}

impl CDrop for CMatcherRewriteMode {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl CReprOf<MatcherRewriteMode> for CMatcherRewriteMode {
    fn c_repr_of(value: MatcherRewriteMode) -> Result<CMatcherRewriteMode, CReprOfError> {
        let variant = match value {
            MatcherRewriteMode::MatcherRewriteAuto => 0,
            MatcherRewriteMode::MatcherRewriteAlways => 1,
            MatcherRewriteMode::MatcherRewriteNever => 2,
        };
        Ok(CMatcherRewriteMode(variant))
    }
}

#[derive(RawPointerConverter, Debug, Clone)]
pub struct CSigmaMatcherConfig {
    pub sigma_label: CLabel,
    pub rewrite_mode: CMatcherRewriteMode,
    pub sigma_allowed_matches: Option<Vec<CLabel>>,
}

impl AsRust<SigmaMatcherConfig> for CSigmaMatcherConfig {
    fn as_rust(&self) -> Result<SigmaMatcherConfig, AsRustError> {
        Ok(SigmaMatcherConfig {
            sigma_label: self.sigma_label.as_rust()?,
            rewrite_mode: self.rewrite_mode.as_rust()?,
            sigma_allowed_matches: self.sigma_allowed_matches.clone(),
        })
    }
}

impl CDrop for CSigmaMatcherConfig {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl CReprOf<SigmaMatcherConfig> for CSigmaMatcherConfig {
    fn c_repr_of(input: SigmaMatcherConfig) -> Result<Self, CReprOfError> {
        Ok(CSigmaMatcherConfig {
            sigma_label: <Label as CReprOf<_>>::c_repr_of(input.sigma_label)?,
            rewrite_mode: CMatcherRewriteMode::c_repr_of(input.rewrite_mode)?,
            sigma_allowed_matches: input.sigma_allowed_matches,
        })
    }
}

#[derive(RawPointerConverter, Debug, Clone, Default)]
pub struct CMatcherConfig {
    pub sigma_matcher_config: Option<CSigmaMatcherConfig>,
}

impl AsRust<MatcherConfig> for CMatcherConfig {
    fn as_rust(&self) -> Result<MatcherConfig, AsRustError> {
        if let Some(v) = &self.sigma_matcher_config {
            Ok(MatcherConfig {
                sigma_matcher_config: Some(v.as_rust()?),
            })
        } else {
            Ok(MatcherConfig {
                sigma_matcher_config: None,
            })
        }
    }
}

impl CDrop for CMatcherConfig {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        self.sigma_matcher_config
            .as_mut()
            .map(|v| v.do_drop())
            .transpose()?;
        Ok(())
    }
}

impl CReprOf<MatcherConfig> for CMatcherConfig {
    fn c_repr_of(input: MatcherConfig) -> Result<Self, CReprOfError> {
        if let Some(v) = input.sigma_matcher_config {
            Ok(Self {
                sigma_matcher_config: Some(CReprOf::c_repr_of(v)?),
            })
        } else {
            Ok(Self {
                sigma_matcher_config: None,
            })
        }
    }
}

#[derive(AsRust, CReprOf, CDrop, RawPointerConverter, Debug)]
#[target_type(ComposeConfig)]
pub struct CComposeConfig {
    pub compose_filter: CComposeFilterEnum,
    pub connect: bool,
    pub matcher1_config: CMatcherConfig,
    pub matcher2_config: CMatcherConfig,
}

#[derive(Debug)]
#[repr(C)]
pub struct CIntArray {
    pub data: *const u32,
    pub size: usize,
}

impl<'a> From<&'a [u32]> for CIntArray {
    fn from(array: &[u32]) -> Self {
        Self {
            size: array.len(),
            data: array.as_ptr(),
        }
    }
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_matcher_config_new(
    sigma_label: libc::size_t,
    rewrite_mode: libc::size_t,
    sigma_allowed_matches: CIntArray,
    config: *mut *const CMatcherConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let sigma_allowed_matches = unsafe {
            std::slice::from_raw_parts(sigma_allowed_matches.data, sigma_allowed_matches.size)
                .to_vec()
        };
        let sigma_allowed_matches = sigma_allowed_matches
            .iter()
            .map(|v| *v as CLabel)
            .collect::<Vec<_>>();
        let sigma_allowed_matches = if sigma_allowed_matches.is_empty() {
            None
        } else {
            Some(sigma_allowed_matches)
        };
        let matcher_config = CMatcherConfig {
            sigma_matcher_config: Some(CSigmaMatcherConfig {
                sigma_label: sigma_label as CLabel,
                rewrite_mode: CMatcherRewriteMode(rewrite_mode),
                sigma_allowed_matches,
            }),
        };

        unsafe { *config = matcher_config.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_compose_config_new(
    compose_filter: libc::size_t,
    connect: bool,
    matcher1_config: *const CMatcherConfig,
    matcher2_config: *const CMatcherConfig,
    config: *mut *const CComposeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let matcher1_config = if matcher1_config.is_null() {
            CMatcherConfig::default()
        } else {
            unsafe {
                <CMatcherConfig as ffi_convert::RawBorrow<CMatcherConfig>>::raw_borrow(
                    matcher1_config,
                )?
            }
            .clone()
        };

        let matcher2_config = if matcher2_config.is_null() {
            CMatcherConfig::default()
        } else {
            unsafe {
                <CMatcherConfig as ffi_convert::RawBorrow<CMatcherConfig>>::raw_borrow(
                    matcher2_config,
                )?
            }
            .clone()
        };

        let compose_config = CComposeConfig {
            matcher1_config,
            matcher2_config,
            compose_filter: CComposeFilterEnum(compose_filter),
            connect,
        };
        unsafe { *config = compose_config.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_matcher_config_destroy(
    ptr: *mut CMatcherConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(ptr) });
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_compose_config_destroy(
    ptr: *mut CComposeConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(ptr) });
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_compose(
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

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_compose_with_config(
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
