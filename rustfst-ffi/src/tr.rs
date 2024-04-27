use crate::{wrap, CLabel, CStateId, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::prelude::{StateId, Tr};
use rustfst::semirings::TropicalWeight;
use rustfst::Semiring;

#[derive(Debug)]
#[repr(C)]
#[derive(CReprOf, AsRust, CDrop, RawPointerConverter)]
#[target_type(Tr::<TropicalWeight>)]
pub struct CTr {
    /// Input label.
    pub ilabel: CLabel,
    /// Output label.
    pub olabel: CLabel,
    /// Weight.
    pub weight: CTropicalWeight,
    /// ID of the next state.
    pub nextstate: CStateId,
}

#[derive(Debug)]
#[repr(C)]
#[derive(CDrop, RawPointerConverter)]
pub struct CTropicalWeight {
    value: libc::c_float,
}

impl CReprOf<TropicalWeight> for CTropicalWeight {
    fn c_repr_of(input: TropicalWeight) -> Result<Self, CReprOfError> {
        Ok(Self {
            value: input.take_value(),
        })
    }
}

impl AsRust<TropicalWeight> for CTropicalWeight {
    fn as_rust(&self) -> Result<TropicalWeight, AsRustError> {
        Ok(self.value.into())
    }
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_new(
    ilabel: CLabel,
    olabel: CLabel,
    weight: libc::c_float,
    nextstate: CStateId,
    new_struct: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = CTr {
            ilabel,
            olabel,
            weight: CTropicalWeight { value: weight },
            nextstate,
        };
        let raw_pointer: *mut CTr = Box::into_raw(Box::new(tr));
        unsafe { *new_struct = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_ilabel(tr: *const CTr, ilabel: *mut CLabel) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? };
        let ilabel_val = tr.ilabel;
        unsafe { *ilabel = ilabel_val }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_set_ilabel(tr: *mut CTr, ilabel: *const CLabel) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = &mut unsafe { <CTr as ffi_convert::RawBorrowMut<CTr>>::raw_borrow_mut(tr)? };
        tr.ilabel = ilabel as CLabel;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_olabel(tr: *const CTr, olabel: *mut CLabel) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? };
        let olabel_val = tr.olabel;
        unsafe { *olabel = olabel_val }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_set_olabel(tr: *mut CTr, olabel: *const CLabel) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = &mut unsafe { <CTr as ffi_convert::RawBorrowMut<CTr>>::raw_borrow_mut(tr)? };
        tr.olabel = olabel as CLabel;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_weight(
    tr: *const CTr,
    weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? };
        let weight_val = *tr.weight.as_rust()?.value();
        unsafe { *weight = weight_val }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_set_weight(tr: *mut CTr, weight: libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = &mut unsafe { <CTr as ffi_convert::RawBorrowMut<CTr>>::raw_borrow_mut(tr)? };
        let tropical_weight = TropicalWeight::new(weight);
        tr.weight = CTropicalWeight::c_repr_of(tropical_weight)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_next_state(
    tr: *const CTr,
    next_state: *mut CStateId,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? };
        let next_state_val = tr.nextstate;
        unsafe { *next_state = next_state_val }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_set_next_state(
    tr: *mut CTr,
    next_state: *const CStateId,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let tr = &mut unsafe { <CTr as ffi_convert::RawBorrowMut<CTr>>::raw_borrow_mut(tr)? };
        tr.nextstate = next_state as StateId;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn tr_delete(tr_ptr: *mut CTr) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if tr_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(tr_ptr) });
        Ok(())
    })
}
