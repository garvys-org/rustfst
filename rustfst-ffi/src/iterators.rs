use crate::fst::CFst;
use crate::tr::CTr;
use crate::{get, get_mut, wrap, RUSTFST_FFI_RESULT};
use anyhow::{anyhow, Result};
use ffi_convert::*;
use rustfst::fst_traits::CoreFst;
use rustfst::prelude::{StateId, Tr, TropicalWeight, VectorFst};

#[derive(Debug)]
pub struct TrsIterator<'a, F: CoreFst<TropicalWeight>> {
    fst: &'a F,
    state: StateId,
    index: usize,
}

impl<'a> TrsIterator<'a, VectorFst<TropicalWeight>> {
    fn done(&self) -> bool {
        self.fst
            .get_trs(self.state)
            .map(|it| it.len() == self.index)
            .unwrap_or(true)
    }

    fn reset(&mut self) {
        self.index = 0
    }
}

impl<'a> Iterator for TrsIterator<'a, VectorFst<TropicalWeight>> {
    type Item = Tr<TropicalWeight>;
    fn next(&mut self) -> Option<Self::Item> {
        let item = self
            .fst
            .get_trs(self.state)
            .map(|it| it.get(self.index).map(|it| it.clone()))
            .ok()
            .flatten();
        self.index += 1;
        item
    }
}

pub struct CTrsIterator<'a>(pub(crate) TrsIterator<'a, VectorFst<TropicalWeight>>);

impl<'a> RawPointerConverter<CTrsIterator<'a>> for CTrsIterator<'a> {
    fn into_raw_pointer(self) -> *const CTrsIterator<'a> {
        Box::into_raw(Box::new(self)) as _
    }
    fn into_raw_pointer_mut(self) -> *mut CTrsIterator<'a> {
        Box::into_raw(Box::new(self))
    }

    unsafe fn from_raw_pointer(
        input: *const CTrsIterator<'a>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input as _))
        }
    }

    unsafe fn from_raw_pointer_mut(
        input: *mut CTrsIterator<'a>,
    ) -> Result<Self, UnexpectedNullPointerError> {
        if input.is_null() {
            Err(UnexpectedNullPointerError)
        } else {
            Ok(*Box::from_raw(input))
        }
    }
}

#[no_mangle]
pub extern "C" fn trs_iterator_new(
    fst_ptr: *mut CFst,
    state_id: libc::size_t,
    iter_ptr: *mut *const CTrsIterator,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let trs_iterator = TrsIterator {
            fst,
            state: state_id,
            index: 0,
        };
        let raw_ptr = CTrsIterator(trs_iterator).into_raw_pointer();
        unsafe { *iter_ptr = raw_ptr };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_iterator_next(
    iter_ptr: *mut CTrsIterator,
    tr_ptr: *mut *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CTrsIterator, iter_ptr);
        let res = trs_iter
            .next()
            .ok_or_else(|| anyhow!("Iteration is done!"))?;
        println!("{:?}", trs_iter);
        let ctr = Box::into_raw(Box::new(CTr::c_repr_of(res.clone())?));
        unsafe { *tr_ptr = ctr };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_iterator_done(
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

#[no_mangle]
pub extern "C" fn trs_iterator_reset(iter_ptr: *mut CTrsIterator) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let trs_iter = get_mut!(CTrsIterator, iter_ptr);
        trs_iter.reset();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn trs_iterator_destroy(iter_ptr: *mut CTrsIterator) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if iter_ptr.is_null() {
            return Ok(());
        }

        unsafe {
            Box::from_raw(iter_ptr);
        }
        Ok(())
    })
}
