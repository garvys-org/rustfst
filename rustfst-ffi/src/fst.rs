use crate::symbol_table::CSymbolTable;
use crate::tr::CTr;
use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};

use anyhow::anyhow;
use ffi_convert::*;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{CoreFst, ExpandedFst, Fst, MutableFst, SerializableFst};
use rustfst::semirings::TropicalWeight;
use rustfst::DrawingConfig;
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
pub extern "C" fn fst_add_state(fst: *mut CFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let res = fst.add_state();
        unsafe { *state = res }
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
    mut input_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.input_symbols()
            .map(|it| {
                let symt = CSymbolTable(it.clone());
                unsafe { *input_symt = symt }
            })
            .unwrap_or_else(|| input_symt = std::ptr::null_mut());
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
    mut output_symt: *mut CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.output_symbols()
            .map(|it| {
                let symt = CSymbolTable(it.clone());
                unsafe { *output_symt = symt }
            })
            .unwrap_or_else(|| output_symt = std::ptr::null_mut());
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
    mut final_weight: *mut libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.final_weight(state_id)?
            .map(|it| unsafe { *final_weight = *it.value() })
            .unwrap_or_else(|| final_weight = std::ptr::null_mut());
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
pub extern "C" fn fst_start(fst: *const CFst, mut state: *mut CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        fst.start()
            .map(|it| unsafe { *state = it })
            .unwrap_or_else(|| state = std::ptr::null_mut());
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
pub extern "C" fn fst_draw(
    fst_ptr: *mut CFst,
    isyms: *const CSymbolTable,
    osyms: *const CSymbolTable,
    fname: *const libc::c_char,
    title: *const libc::c_char,
    acceptor: libc::size_t,
    width: libc::c_float,
    height: libc::c_float,
    portrait: libc::size_t,
    vertical: libc::size_t,
    ranksep: libc::c_float,
    nodesep: libc::c_float,
    fontsize: libc::size_t,
    show_weight_one: libc::size_t,
    print_weight: libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst_ptr);

        if !isyms.is_null() {
            let isymt = get!(CSymbolTable, isyms);
            fst.set_input_symbols(isymt.clone());
        }

        if !osyms.is_null() {
            let osymt = get!(CSymbolTable, osyms);
            fst.set_output_symbols(osymt.clone());
        }

        let drawing_config = DrawingConfig {
            vertical: if vertical > 0 { true } else { false },
            size: if (width > 0.0) && (height > 0.0) {
                Some((width, height))
            } else {
                None
            },
            title: unsafe { CStr::from_ptr(title).as_rust()? },
            portrait: if portrait > 0 { true } else { false },
            ranksep: if ranksep > 0.0 { Some(ranksep) } else { None },
            nodesep: if nodesep > 0.0 { Some(nodesep) } else { None },
            fontsize: fontsize as u32,
            acceptor: if acceptor > 0 { true } else { false },
            show_weight_one: if show_weight_one > 0 { true } else { false },
            print_weight: if print_weight > 0 { true } else { false },
        };

        fst.draw(unsafe { CStr::from_ptr(fname).as_rust()? }, &drawing_config)?;

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
