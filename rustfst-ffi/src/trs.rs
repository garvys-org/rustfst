use crate::tr::CTr;
use crate::{wrap, RUSTFST_FFI_RESULT};
use ffi_convert::*;
use rustfst::prelude::TrsVec;
use rustfst::semirings::TropicalWeight;
use rustfst::Trs;

use anyhow::Result;

#[derive(RawPointerConverter)]
pub struct CTrs(TrsVec<TropicalWeight>);

macro_rules! get_trs_mut {
    ($typ:ty,$opaque:ident) => {{
        &mut unsafe { <$typ as ffi_convert::RawBorrowMut<$typ>>::raw_borrow_mut($opaque) }?.0
    }};
}

macro_rules! get_trs {
    ($typ:ty,$opaque:ident) => {{
        &unsafe { <$typ as ffi_convert::RawBorrow<$typ>>::raw_borrow($opaque) }?.0
    }};
}

#[no_mangle]
pub extern "C" fn trs_vec_new(new_struct: *mut *const CTrs) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs: TrsVec<TropicalWeight> = TrsVec::default();
        let raw_pointer = CTrs(trs).into_raw_pointer();
        unsafe { *new_struct = raw_pointer };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_vec_remove(
    trs: *mut CTrs,
    index: libc::size_t,
    removed_tr_ptr: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get_trs_mut!(CTrs, trs);
        let removed_tr = trs.remove(index);
        let ctr = Box::into_raw(Box::new(CTr::c_repr_of(removed_tr)?));
        unsafe { *removed_tr_ptr = ctr };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_vec_push(trs: *mut CTrs, new_tr: *const CTr) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get_trs_mut!(CTrs, trs);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(new_tr)? };
        trs.push(tr.as_rust()?);
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_vec_shallow_clone(
    trs: *const CTrs,
    cloned_trs_ptr: *mut *const CTrs,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs = get_trs!(CTrs, trs);
        let cloned_trs = trs.shallow_clone();
        let raw_pointer = CTrs(cloned_trs).into_raw_pointer();
        unsafe { *cloned_trs_ptr = raw_pointer };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_vec_delete(trs_ptr: *mut CTrs) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if trs_ptr.is_null() {
            return Ok(());
        }

        unsafe {
            Box::from_raw(trs_ptr);
        }
        Ok(())
    })
}
