use crate::tr::CTr;
use crate::{get, get_mut, wrap, RUSTFST_FFI_RESULT};
use std::ffi::CString;

use anyhow::Result;
use ffi_convert::*;
use rustfst::prelude::TrsVec;
use rustfst::semirings::TropicalWeight;
use rustfst::Trs;

#[derive(RawPointerConverter)]
pub struct CTrs(pub(crate) TrsVec<TropicalWeight>);

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_new(new_struct: *mut *const CTrs) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs: TrsVec<TropicalWeight> = TrsVec::default();
        let raw_pointer = CTrs(trs).into_raw_pointer();
        unsafe { *new_struct = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_remove(
    trs: *mut CTrs,
    index: libc::size_t,
    removed_tr_ptr: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get_mut!(CTrs, trs);
        let removed_tr = trs.remove(index);
        let ctr = Box::into_raw(Box::new(CTr::c_repr_of(removed_tr)?));
        unsafe { *removed_tr_ptr = ctr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_push(trs: *mut CTrs, new_tr: *const CTr) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get_mut!(CTrs, trs);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(new_tr)? };
        trs.push(tr.as_rust()?);
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_shallow_clone(
    trs: *const CTrs,
    cloned_trs_ptr: *mut *const CTrs,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get!(CTrs, trs);
        let cloned_trs = trs.shallow_clone();
        let raw_pointer = CTrs(cloned_trs).into_raw_pointer();
        unsafe { *cloned_trs_ptr = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_len(
    trs: *const CTrs,
    num_trs: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get!(CTrs, trs);
        unsafe { *num_trs = trs.len() as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// `trs` pointer should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_display(
    trs: *const CTrs,
    string: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get!(CTrs, trs);
        let trs_display = format!("{:?}", trs);
        unsafe {
            *string = CString::c_repr_of(trs_display)?.into_raw_pointer() as *const libc::c_char
        };
        Ok(())
    })
}

/// # Safety
///
/// Should never happen.
#[no_mangle]
pub unsafe extern "C" fn trs_vec_delete(trs_ptr: *mut CTrs) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if trs_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(trs_ptr) });
        Ok(())
    })
}
