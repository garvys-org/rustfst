use std::iter::Peekable;

use anyhow::{anyhow, Result};
use ffi_convert::{RawPointerConverter, UnexpectedNullPointerError};

use rustfst::fst_impls::VectorFst;
use rustfst::prelude::{Fst, StringPathsIterator};
use rustfst::semirings::TropicalWeight;

use crate::fst::as_fst;
use crate::fst::CFst;
use crate::string_path::CStringPath;
use crate::{get, get_mut, wrap, RUSTFST_FFI_RESULT};

pub struct CStringPathsIterator<'a>(
    pub(crate) Peekable<StringPathsIterator<'a, TropicalWeight, VectorFst<TropicalWeight>>>,
);

impl<'a> RawPointerConverter<CStringPathsIterator<'a>> for CStringPathsIterator<'a> {
    fn into_raw_pointer(self) -> *const CStringPathsIterator<'a> {
        Box::into_raw(Box::new(self)) as _
    }
    fn into_raw_pointer_mut(self) -> *mut CStringPathsIterator<'a> {
        Box::into_raw(Box::new(self))
    }

    unsafe fn from_raw_pointer(
        input: *const CStringPathsIterator<'a>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input as _))
        }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CStringPathsIterator<'a>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input))
        }
    }
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_paths_iterator_new(
    fst: *const CFst,
    res_iterator: *mut *const CStringPathsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let fst: &VectorFst<_> = as_fst!(VectorFst<TropicalWeight>, fst);
        let it = fst.string_paths_iter()?.peekable();
        let raw_pointer = CStringPathsIterator(it).into_raw_pointer();
        unsafe { *res_iterator = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_paths_iterator_next(
    iter_ptr: *mut CStringPathsIterator,
    string_path_ptr: *mut *const CStringPath,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let string_paths_iter = get_mut!(CStringPathsIterator, iter_ptr);
        string_paths_iter
            .next()
            .map(|string_path| {
                let ctr = CStringPath(string_path).into_raw_pointer();
                unsafe { *string_path_ptr = ctr };
                Ok(())
            })
            .unwrap_or_else(|| -> Result<()> {
                unsafe { *string_path_ptr = std::ptr::null_mut() };
                Ok(())
            })?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_paths_iterator_done(
    iter_ptr: *mut CStringPathsIterator,
    done: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let string_paths_iter = get_mut!(CStringPathsIterator, iter_ptr);
        let res = string_paths_iter.peek().is_none();
        unsafe { *done = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn string_paths_iterator_destroy(
    iter_ptr: *mut CStringPathsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(iter_ptr) });
        Ok(())
    })
}
