#![allow(clippy::single_component_path_imports)]

pub mod algorithms;
pub mod fst;
pub mod iterators;
pub mod string_path;
pub mod string_paths_iterator;
pub mod symbol_table;
pub mod tr;
pub mod trs;

use std::cell::RefCell;
use std::ffi::CString;
use std::sync::Arc;

use anyhow::Result;
use ffi_convert::{CReprOf, RawPointerConverter};

#[cfg(feature = "rustfst-state-label-u32")]
pub type CLabel = libc::c_uint;
#[cfg(not(feature = "rustfst-state-label-u32"))]
pub type CLabel = libc::size_t;

#[cfg(feature = "rustfst-state-label-u32")]
pub type CStateId = libc::c_uint;
#[cfg(not(feature = "rustfst-state-label-u32"))]
pub type CStateId = libc::size_t;

#[repr(C)]
#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Eq)]
pub enum RUSTFST_FFI_RESULT {
    /// The function returned successfully
    RUSTFST_FFI_RESULT_OK = 0,
    /// The function returned an error
    RUSTFST_FFI_RESULT_KO = 1,
}

thread_local! {
    pub(crate) static LAST_ERROR: RefCell<Option<String>> = const { RefCell::new(None) };
}

pub fn wrap<F: FnOnce() -> Result<()>>(func: F) -> RUSTFST_FFI_RESULT {
    match func() {
        Ok(_) => RUSTFST_FFI_RESULT::RUSTFST_FFI_RESULT_OK,
        Err(e) => {
            let msg = format!("{:#?}", e);
            if std::env::var("AMSTRAM_FFI_ERROR_STDERR").is_ok() {
                eprintln!("{}", msg);
            }
            LAST_ERROR.with(|p| *p.borrow_mut() = Some(msg));
            RUSTFST_FFI_RESULT::RUSTFST_FFI_RESULT_KO
        }
    }
}

/// # Safety
///
/// Should never happen
#[no_mangle]
pub unsafe extern "C" fn rustfst_ffi_get_last_error(
    error: *mut *mut ::libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(move || {
        LAST_ERROR.with(|msg| {
            let string = msg
                .borrow_mut()
                .take()
                .unwrap_or_else(|| "No error message".to_string());
            let result: *const ::libc::c_char =
                std::ffi::CString::c_repr_of(string)?.into_raw_pointer();
            unsafe { *error = result as _ }
            Ok(())
        })
    })
}

#[allow(clippy::missing_safety_doc)]
#[no_mangle]
pub unsafe extern "C" fn rustfst_destroy_string(string: *mut libc::c_char) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        CString::drop_raw_pointer(string)?;
        Ok(())
    })
}

macro_rules! get_mut {
    ($typ:ty,$opaque:ident) => {{
        &mut unsafe { <$typ as ffi_convert::RawBorrowMut<$typ>>::raw_borrow_mut($opaque) }?.0
    }};
}

macro_rules! get {
    ($typ:ty,$opaque:ident) => {{
        &unsafe { <$typ as ffi_convert::RawBorrow<$typ>>::raw_borrow($opaque) }?.0
    }};
}

use crate::symbol_table::CSymbolTable;
pub(crate) use get;
pub(crate) use get_mut;
use rustfst::SymbolTable;

pub(crate) fn get_symt(symt: *const CSymbolTable) -> Result<Option<&'static Arc<SymbolTable>>> {
    if symt.is_null() {
        return Ok(None);
    }
    Ok(Some(get!(CSymbolTable, symt)))
}
