use crate::fst::CFst;
use crate::symbol_table::CSymbolTable;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};
use anyhow::{anyhow, Context, Result};
use ffi_convert::*;
use rustfst::prelude::{Label, Semiring, TropicalWeight, VectorFst};
use rustfst::utils::{acceptor, transducer};
use std::ffi::CStr;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn utils_string_to_acceptor(
    astring: *const libc::c_char,
    symbol_table: *mut CSymbolTable,
    weight: libc::c_float,
    fst_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let astring: String = unsafe { CStr::from_ptr(astring) }.as_rust()?;
        let symt = get_mut!(CSymbolTable, symbol_table);
        let labels = astring
            .split(' ')
            .map(|sym| -> Result<Label> {
                symt.get_label(sym)
                    .with_context(|| anyhow!("Could not retrieve symbol {:?} in symbol table", sym))
            })
            .collect::<Result<Vec<Label>>>()?;
        let acceptor_fst: VectorFst<TropicalWeight> =
            acceptor(labels.as_slice(), TropicalWeight::new(weight));
        unsafe { *fst_ptr = CFst(Box::new(acceptor_fst)).into_raw_pointer() }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn utils_string_to_transducer(
    istring: *const libc::c_char,
    ostring: *const libc::c_char,
    isymt: *mut CSymbolTable,
    osymt: *mut CSymbolTable,
    weight: libc::c_float,
    fst_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let istring: String = unsafe { CStr::from_ptr(istring) }.as_rust()?;
        let ostring: String = unsafe { CStr::from_ptr(ostring) }.as_rust()?;
        let isymt = get_mut!(CSymbolTable, isymt);
        let osymt = get_mut!(CSymbolTable, osymt);
        let ilabels = istring
            .split(' ')
            .map(|sym| -> Result<Label> {
                isymt
                    .get_label(sym)
                    .with_context(|| anyhow!("Could not retrieve symbol {:?} in symbol table", sym))
            })
            .collect::<Result<Vec<Label>>>()?;
        let olabels = ostring
            .split(' ')
            .map(|sym| -> Result<Label> {
                osymt
                    .get_label(sym)
                    .with_context(|| anyhow!("Could not retrieve symbol {:?} in symbol table", sym))
            })
            .collect::<Result<Vec<Label>>>()?;
        let transducer_fst: VectorFst<TropicalWeight> = transducer(
            ilabels.as_slice(),
            olabels.as_slice(),
            TropicalWeight::new(weight),
        );
        unsafe { *fst_ptr = CFst(Box::new(transducer_fst)).into_raw_pointer() }
        Ok(())
    })
}
