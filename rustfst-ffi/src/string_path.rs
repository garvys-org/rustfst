use ffi_convert::{CReprOf, RawPointerConverter};
use std::ffi::CString;

use rustfst::semirings::TropicalWeight;
use rustfst::{Semiring, StringPath};

use crate::{get, wrap, RUSTFST_FFI_RESULT};

#[derive(RawPointerConverter)]
pub struct CStringPath(pub(crate) StringPath<TropicalWeight>);

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_path_destroy(iter_ptr: *mut CStringPath) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(iter_ptr) });
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_path_weight(
    c_string_path: *const CStringPath,
    weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let string_path = get!(CStringPath, c_string_path);
        let weight_val = *string_path.weight().value();
        unsafe { *weight = weight_val }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_path_istring(
    c_string_path: *const CStringPath,
    c_istring: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let string_path = get!(CStringPath, c_string_path);
        let istring = string_path.istring()?;
        unsafe {
            *c_istring = CString::c_repr_of(istring)?.into_raw_pointer() as *const libc::c_char
        }

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_path_ostring(
    c_string_path: *const CStringPath,
    c_ostring: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let string_path = get!(CStringPath, c_string_path);
        let ostring = string_path.ostring()?;
        unsafe {
            *c_ostring = CString::c_repr_of(ostring)?.into_raw_pointer() as *const libc::c_char
        }

        Ok(())
    })
}
