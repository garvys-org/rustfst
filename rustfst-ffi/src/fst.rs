use crate::symbol_table::CSymbolTable;
use crate::tr::CTr;
use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};

use anyhow::anyhow;
use ffi_convert::*;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;
use rustfst::Semiring;

use std::ffi::CStr;

#[derive(RawPointerConverter)]
pub struct CFst(pub(crate) VectorFst<TropicalWeight>);

#[no_mangle]
pub extern "C" fn fst_new(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = VectorFst::<TropicalWeight>::new();
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_from_path(
    ptr: *mut *const CFst,
    path: *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let fst = VectorFst::<TropicalWeight>::read(&path)?;
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_write_file(
    fst: *const CFst,
    path: *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        fst.write(&path)?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_add_tr(
    fst: *mut CFst,
    state: CStateId,
    tr: *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? }.as_rust()?;
        fst.add_tr(state, tr)?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_num_trs(
    fst: *const CFst,
    state: CStateId,
    num_trs: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.num_trs(state)?;
        unsafe { *num_trs = res };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_add_state(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.add_state();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_num_states(
    fst: *const CFst,
    num_states: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.num_states();
        unsafe { *num_states = res };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_delete_states(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.del_all_states();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_input_symbols(
    fst: *const CFst,
    input_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst
            .input_symbols()
            .ok_or_else(|| anyhow!("No input symbols set"))?;
        let symt = CSymbolTable(res.clone());
        unsafe { *input_symt = symt };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_set_input_symbols(
    fst: *mut CFst,
    symt: *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let symt = get!(CSymbolTable, symt);
        fst.set_input_symbols(symt.clone());
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_unset_input_symbols(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.take_input_symbols();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_output_symbols(
    fst: *const CFst,
    output_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst
            .output_symbols()
            .ok_or_else(|| anyhow!("No output symbols set"))?;
        let symt = CSymbolTable(res.clone());
        unsafe { *output_symt = symt };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_set_output_symbols(
    fst: *mut CFst,
    symt: *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let symt = get!(CSymbolTable, symt);
        fst.set_output_symbols(symt.clone());
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_unset_output_symbols(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.take_output_symbols();
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_set_final(
    fst: *mut CFst,
    state: CStateId,
    weight: libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.set_final(state, TropicalWeight::new(weight as f32))?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_is_final(
    fst: *const CFst,
    state: CStateId,
    is_final: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst.is_final(state)?;
        unsafe { *is_final = res as usize }
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_final_weight(
    fst: *const CFst,
    state_id: CStateId,
    final_weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst
            .final_weight(state_id)?
            .ok_or_else(|| anyhow!("State '{:?}' is NOT final", state_id))?;
        unsafe { *final_weight = *res.value() };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_set_start(fst: *mut CFst, state: CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        fst.set_start(state)?;
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_start(fst: *const CFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let res = fst
            .start()
            .ok_or_else(|| anyhow!("FST has no start state"))?;
        unsafe { *state = res }
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_equals(
    fst: *const CFst,
    other_fst: *const CFst,
    is_equal: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let other_fst = get!(CFst, other_fst);
        let res = fst.eq(other_fst);
        unsafe { *is_equal = res as usize }
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_weight_one(weight_one: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::one();
        unsafe { *weight_one = *weight.value() };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_weight_zero(weight_zero: *mut libc::c_float) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let weight = TropicalWeight::zero();
        unsafe { *weight_zero = *weight.value() };
        Ok(())
    })
}

/// drop impl
#[no_mangle]
pub extern "C" fn fst_destroy(fst_ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if fst_ptr.is_null() {
            return Ok(());
        }

        unsafe {
            Box::from_raw(fst_ptr);
        }
        Ok(())
    })
}
