pub mod tr;
use std::cell::RefCell;

use anyhow::Result;
use ffi_convert::{CReprOf, RawPointerConverter};

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
    pub(crate) static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
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

#[no_mangle]
pub extern "C" fn rustfst_ffi_get_last_error(
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
