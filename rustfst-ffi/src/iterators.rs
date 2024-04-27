use crate::fst::{CFst, CVecFst};
use crate::tr::CTr;
use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};
use anyhow::Result;
use ffi_convert::*;
use rustfst::fst_traits::MutableFst;
use rustfst::prelude::{StateIterator, Tr, TropicalWeight, TrsVec};
use rustfst::trs_iter_mut::TrsIterMut;
use std::iter::Peekable;
use std::ops::Range;

#[derive(Debug)]
pub struct TrsIterator {
    trs: TrsVec<TropicalWeight>,
    index: usize,
}

impl TrsIterator {
    fn done(&self) -> bool {
        self.trs.len() == self.index
    }

    fn reset(&mut self) {
        self.index = 0
    }
}

impl Iterator for TrsIterator {
    type Item = Tr<TropicalWeight>;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self.trs.get(self.index).cloned();
        self.index += 1;
        item
    }
}

#[derive(RawPointerConverter)]
pub struct CTrsIterator(pub(crate) TrsIterator);

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_iterator_new(
    fst_ptr: *mut CFst,
    state_id: CStateId,
    mut iter_ptr: *mut *const CTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        fst.fst_get_trs(state_id)
            .map(|trs| {
                let raw_ptr = {
                    let trs_iterator = TrsIterator { trs, index: 0 };
                    CTrsIterator(trs_iterator).into_raw_pointer()
                };

                unsafe { *iter_ptr = raw_ptr };
            })
            .unwrap_or_else(|_| iter_ptr = std::ptr::null_mut());

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_iterator_next(
    iter_ptr: *mut CTrsIterator,
    mut tr_ptr: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CTrsIterator, iter_ptr);
        trs_iter
            .next()
            .map(|tr| {
                let ctr = Box::into_raw(Box::new(CTr::c_repr_of(tr)?));
                unsafe { *tr_ptr = ctr };
                Ok(())
            })
            .unwrap_or_else(|| -> Result<()> {
                tr_ptr = std::ptr::null_mut();
                Ok(())
            })?;

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_iterator_done(
    iter_ptr: *const CTrsIterator,
    done: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get!(CTrsIterator, iter_ptr);
        let res = trs_iter.done();
        unsafe { *done = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_iterator_reset(iter_ptr: *mut CTrsIterator) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CTrsIterator, iter_ptr);
        trs_iter.reset();
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn trs_iterator_destroy(iter_ptr: *mut CTrsIterator) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(iter_ptr) });
        Ok(())
    })
}

pub struct MutTrsIterator<'a> {
    trs: TrsIterMut<'a, TropicalWeight>,
    index: usize,
}

impl<'a> MutTrsIterator<'a> {
    pub fn done(&self) -> bool {
        self.trs.len() == self.index
    }

    pub fn next(&mut self) {
        self.index += 1
    }

    pub fn value(&self) -> Option<Tr<TropicalWeight>> {
        self.trs.get(self.index).cloned()
    }

    pub fn set_value(&mut self, tr: Tr<TropicalWeight>) -> Result<()> {
        self.trs.set_tr(self.index, tr)
    }

    pub fn reset(&mut self) {
        self.index = 0
    }
}

pub struct CMutTrsIterator<'a>(pub(crate) MutTrsIterator<'a>);

impl<'a> RawPointerConverter<CMutTrsIterator<'a>> for CMutTrsIterator<'a> {
    fn into_raw_pointer(self) -> *const CMutTrsIterator<'a> {
        Box::into_raw(Box::new(self)) as _
    }
    fn into_raw_pointer_mut(self) -> *mut CMutTrsIterator<'a> {
        Box::into_raw(Box::new(self))
    }

    unsafe fn from_raw_pointer(
        input: *const CMutTrsIterator<'a>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input as _))
        }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CMutTrsIterator<'a>,
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
pub unsafe extern "C" fn mut_trs_iterator_new(
    fst_ptr: *mut CVecFst,
    state_id: CStateId,
    mut iter_ptr: *mut *const CMutTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CVecFst, fst_ptr);
        fst.tr_iter_mut(state_id)
            .map(|trs| {
                let raw_ptr = {
                    let trs_iterator = MutTrsIterator { trs, index: 0 };
                    CMutTrsIterator(trs_iterator).into_raw_pointer()
                };

                unsafe { *iter_ptr = raw_ptr };
            })
            .unwrap_or_else(|_| iter_ptr = std::ptr::null_mut());

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_next(
    iter_ptr: *mut CMutTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CMutTrsIterator, iter_ptr);
        trs_iter.next();
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_value(
    iter_ptr: *mut CMutTrsIterator,
    mut tr_ptr: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CMutTrsIterator, iter_ptr);
        trs_iter
            .value()
            .map(|tr| {
                let ctr = Box::into_raw(Box::new(CTr::c_repr_of(tr)?));
                unsafe { *tr_ptr = ctr };
                Ok(())
            })
            .unwrap_or_else(|| -> Result<()> {
                tr_ptr = std::ptr::null_mut();
                Ok(())
            })?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_set_value(
    iter_ptr: *mut CMutTrsIterator,
    tr_ptr: *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CMutTrsIterator, iter_ptr);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr_ptr)? }.as_rust()?;
        trs_iter.set_value(tr)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_done(
    iter_ptr: *const CMutTrsIterator,
    done: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get!(CMutTrsIterator, iter_ptr);
        let res = trs_iter.done();
        unsafe { *done = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_reset(
    iter_ptr: *mut CMutTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CMutTrsIterator, iter_ptr);
        trs_iter.reset();
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn mut_trs_iterator_destroy(
    iter_ptr: *mut CMutTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(iter_ptr) });
        Ok(())
    })
}

#[derive(RawPointerConverter)]
pub struct CStateIterator(pub(crate) Peekable<Range<CStateId>>);

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn state_iterator_new(
    fst_ptr: *mut CVecFst,
    iter_ptr: *mut *const CStateIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CVecFst, fst_ptr);
        let state_iter = fst.states_iter().peekable();
        let raw_ptr = CStateIterator(state_iter).into_raw_pointer();
        unsafe { *iter_ptr = raw_ptr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn state_iterator_next(
    iter_ptr: *mut CStateIterator,
    mut state: *mut CStateId,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let state_iter = get_mut!(CStateIterator, iter_ptr);
        state_iter
            .next()
            .map(|it| unsafe { *state = it })
            .unwrap_or_else(|| state = std::ptr::null_mut());
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn state_iterator_done(
    iter_ptr: *mut CStateIterator,
    done: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CStateIterator, iter_ptr);
        let res = trs_iter.peek().is_none();
        unsafe { *done = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn state_iterator_destroy(
    iter_ptr: *mut CStateIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(iter_ptr) });
        Ok(())
    })
}
