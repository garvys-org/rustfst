pub mod concat_fst;
pub mod const_fst;
pub mod utils;
pub mod vector_fst;

use crate::symbol_table::CSymbolTable;
use crate::tr::CTr;
use crate::trs::CTrs;
use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};

use anyhow::Result;
use downcast_rs::Downcast;
use ffi_convert::*;
use rustfst::algorithms::concat::ConcatFst;
use rustfst::fst_impls::{ConstFst, VectorFst};
use rustfst::fst_traits::{Fst, MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;
use rustfst::Semiring;
use rustfst::{StateId, SymbolTable, Trs, TrsVec};
use std::ffi::CStr;
use std::sync::Arc;

/// This trait is an alias for the FST trait.
/// It makes the FST trait Boxable and downcastable to one of the supported C Fst structs.
/// This trait allows to share Fst trait methods accross FST types by sharing a common input type in the binded methods.
/// This generic Fst type can then be downcast to the appropriate Fst type (VectorFst, ConstFst, ..) in order to get access to specific methods (add_tr, ..).
pub trait BindableFst: Downcast {
    fn fst_start(&self) -> Option<StateId>;
    fn fst_final_weight(&self, state: StateId) -> Result<Option<TropicalWeight>>;
    fn fst_num_trs(&self, s: StateId) -> Result<usize>;

    #[inline]
    fn fst_is_final(&self, state_id: StateId) -> Result<bool> {
        let w = self.fst_final_weight(state_id)?;
        Ok(w.is_some())
    }

    #[inline]
    fn fst_is_start(&self, state_id: StateId) -> bool {
        Some(state_id) == self.fst_start()
    }

    fn fst_get_trs(&self, state_id: StateId) -> Result<TrsVec<TropicalWeight>>;
    fn fst_input_symbols(&self) -> Option<Arc<SymbolTable>>;
    fn fst_output_symbols(&self) -> Option<Arc<SymbolTable>>;
    fn fst_set_input_symbols(&mut self, symt: Arc<SymbolTable>);
    fn fst_set_output_symbols(&mut self, symt: Arc<SymbolTable>);
    fn fst_take_input_symbols(&mut self) -> Option<Arc<SymbolTable>>;
    fn fst_take_output_symbols(&mut self) -> Option<Arc<SymbolTable>>;
}

downcast_rs::impl_downcast!(BindableFst);

impl<F: Fst<TropicalWeight> + 'static> BindableFst for F {
    fn fst_start(&self) -> Option<StateId> {
        self.start()
    }
    fn fst_final_weight(&self, state: StateId) -> Result<Option<TropicalWeight>> {
        self.final_weight(state)
    }
    fn fst_num_trs(&self, s: StateId) -> Result<usize> {
        self.num_trs(s)
    }
    fn fst_get_trs(&self, state_id: StateId) -> Result<TrsVec<TropicalWeight>> {
        self.get_trs(state_id).map(|it| it.to_trs_vec())
    }
    fn fst_input_symbols(&self) -> Option<Arc<SymbolTable>> {
        self.input_symbols().cloned()
    }
    fn fst_output_symbols(&self) -> Option<Arc<SymbolTable>> {
        self.output_symbols().cloned()
    }
    fn fst_set_input_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.set_input_symbols(symt)
    }
    fn fst_set_output_symbols(&mut self, symt: Arc<SymbolTable>) {
        self.set_output_symbols(symt)
    }
    fn fst_take_input_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.take_input_symbols()
    }
    fn fst_take_output_symbols(&mut self) -> Option<Arc<SymbolTable>> {
        self.take_output_symbols()
    }
}

#[derive(RawPointerConverter)]
pub struct CFst(pub Box<dyn BindableFst>);

#[derive(RawPointerConverter)]
pub struct CVecFst(pub Box<VectorFst<TropicalWeight>>);

#[derive(RawPointerConverter)]
pub struct CConstFst(pub Box<ConstFst<TropicalWeight>>);

#[derive(RawPointerConverter)]
pub struct CConcatFst(pub Box<ConcatFst<TropicalWeight, VectorFst<TropicalWeight>>>);

macro_rules! as_fst {
    ($typ:ty,$fst:ident) => {{
        $fst.downcast_ref::<$typ>()
            .ok_or_else(|| anyhow!("Could not downcast to {} FST", stringify!($typ)))?
    }};
}

macro_rules! as_mut_fst {
    ($typ:ty,$fst:ident) => {{
        $fst.downcast_mut::<$typ>()
            .ok_or_else(|| anyhow!("Could not downcast to {} FST", stringify!($typ)))?
    }};
}

pub(crate) use as_fst;
pub(crate) use as_mut_fst;
//macro_rules! as_const_fst {
//    ($typ:ty,$opaque:ident) => {{
//        &unsafe { <$typ as ffi_convert::RawBorrow<$typ>>::raw_borrow($opaque) }?.0
//    }};
//}

/// Core FST methods
/// As defined in fst_traits

/// Returns the ID of the start state of the wFST if it exists else none
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_start(fst: *const CFst, mut state: *mut CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.fst_start()
            .map(|it| unsafe { *state = it })
            .unwrap_or_else(|| state = std::ptr::null_mut());
        Ok(())
    })
}

/// Retrieves the final weight of a state (if the state is a final one).
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_final_weight(
    fst: *const CFst,
    state_id: CStateId,
    mut final_weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.fst_final_weight(state_id)?
            .map(|it| unsafe { *final_weight = *it.value() })
            .unwrap_or_else(|| final_weight = std::ptr::null_mut());
        Ok(())
    })
}

/// Number of trs leaving a specific state in the wFST.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_num_trs(
    fst: *const CFst,
    state: CStateId,
    num_trs: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.fst_num_trs(state)?;
        unsafe { *num_trs = res };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_get_trs(
    fst: *const CFst,
    state: CStateId,
    trs: *mut *const CTrs,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.fst_get_trs(state)?;
        let trs_vec = CTrs(res).into_raw_pointer();
        unsafe { *trs = trs_vec }
        Ok(())
    })
}

/// Returns whether or not the state with identifier passed as parameters is a final state.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_is_final(
    fst: *const CFst,
    state: CStateId,
    is_final: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.fst_is_final(state)?;
        unsafe { *is_final = res as usize }
        Ok(())
    })
}

/// Returns whether or not the state with identifier passed as parameters is a final state.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_is_start(
    fst: *const CFst,
    state: CStateId,
    is_start: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.fst_is_start(state);
        unsafe { *is_start = res as usize }
        Ok(())
    })
}

// Missing methods for the CoreFst trait:
//- fn properties(&self) -> FstProperties;
//- fn get_trs(&self, state_id: StateId) -> Result<Self::TRS>;
//- fn num_input_epsilons(&self, state: StateId) -> Result<usize>;
//- fn num_output_epsilons(&self, state: StateId) -> Result<usize>;

/// Fst methods
/// As described in fst_traits

/// Retrieves the input `SymbolTable` associated to the Fst.
/// If no SymbolTable has been previously attached then a null pointer is returned.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_input_symbols(
    fst: *const CFst,
    mut input_symt: *mut *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.fst_input_symbols()
            .map(|it| {
                let symt = CSymbolTable(it).into_raw_pointer();
                unsafe { *input_symt = symt }
            })
            .unwrap_or_else(|| input_symt = std::ptr::null_mut());
        Ok(())
    })
}

/// Retrieves the output `SymbolTable` associated to the Fst.
/// If no SymbolTable has been previously attached then a null pointer is returned.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_output_symbols(
    fst: *const CFst,
    mut output_symt: *mut *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.fst_output_symbols()
            .map(|it| {
                let symt = CSymbolTable(it).into_raw_pointer();
                unsafe { *output_symt = symt }
            })
            .unwrap_or_else(|| output_symt = std::ptr::null_mut());
        Ok(())
    })
}

/// Attaches an input `SymbolTable` to the Fst.
/// The `SymbolTable` is not duplicated with the use of Arc.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_set_input_symbols(
    fst: *mut CFst,
    symt: *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let symt = get!(CSymbolTable, symt);
        fst.fst_set_input_symbols(symt.clone());
        Ok(())
    })
}

/// Attaches an output `SymbolTable` to the Fst.
/// The `SymbolTable` is not duplicated with the use of Arc.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_set_output_symbols(
    fst: *mut CFst,
    symt: *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let symt = get!(CSymbolTable, symt);
        fst.fst_set_output_symbols(symt.clone());
        Ok(())
    })
}

/// Removes the input symbol table from the Fst and retrieves it.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_unset_input_symbols(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.fst_take_input_symbols();
        Ok(())
    })
}

/// Removes the output symbol table from the Fst and retrieves it.
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_unset_output_symbols(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.fst_take_output_symbols();
        Ok(())
    })
}

// Missing methods for Fst trait:
//- fn set_symts_from_fst<W2: Semiring, OF: Fst<W2>>(&mut self, other_fst: &OF);
//- fn final_states_iter(&self) -> FinalStatesIterator<W, Self>;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_weight_one(weight_one: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::one();
        unsafe { *weight_one = *weight.value() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_weight_zero(weight_zero: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::zero();
        unsafe { *weight_zero = *weight.value() };
        Ok(())
    })
}

/// drop impl
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn fst_destroy(fst_ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if fst_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(fst_ptr) });
        Ok(())
    })
}
