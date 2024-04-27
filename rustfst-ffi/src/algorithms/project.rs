use anyhow::{anyhow, Result};

use super::EnumConversionError;
use crate::fst::CFst;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::{project, ProjectType};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[derive(RawPointerConverter)]
pub struct CProjectType(usize);

impl AsRust<ProjectType> for CProjectType {
    fn as_rust(&self) -> Result<ProjectType, AsRustError> {
        match self.0 {
            0 => Ok(ProjectType::ProjectInput),
            1 => Ok(ProjectType::ProjectOutput),
            _ => Err(AsRustError::Other(Box::new(EnumConversionError {}))),
        }
    }
}

impl CDrop for CProjectType {
    fn do_drop(&mut self) -> Result<(), CDropError> {
        Ok(())
    }
}

impl CReprOf<ProjectType> for CProjectType {
    fn c_repr_of(value: ProjectType) -> Result<CProjectType, CReprOfError> {
        let variant = match value {
            ProjectType::ProjectInput => 0,
            ProjectType::ProjectOutput => 1,
        };
        Ok(CProjectType(variant))
    }
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_project_type_new(
    project_type: libc::size_t,
    ptr: *mut *const CProjectType,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let project_type = CProjectType(project_type);
        unsafe { *ptr = project_type.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_project(
    ptr: *mut CFst,
    config: *const CProjectType,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let project_type =
            unsafe { <CProjectType as ffi_convert::RawBorrow<CProjectType>>::raw_borrow(config)? };

        project(vec_fst, project_type.as_rust()?);
        Ok(())
    })
}
