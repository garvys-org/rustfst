use super::*;
use crate::get_symt;
use anyhow::{anyhow, format_err};
use ffi_convert::CArray;
use rustfst::fst_traits::ExpandedFst;
use rustfst::DrawingConfig;
use std::ffi::CString;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_new(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = Box::new(VectorFst::new());
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_set_start(fst: *mut CFst, state: CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let c_fst = get_mut!(CFst, fst);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, c_fst);
        vec_fst.set_start(state)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_set_final(
    fst: *mut CFst,
    state: CStateId,
    weight: libc::c_float,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        vec_fst.set_final(state, TropicalWeight::new(weight))?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_add_state(fst: *mut CFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        let res = vec_fst.add_state();
        unsafe { *state = res }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_delete_states(fst: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        vec_fst.del_all_states();
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_add_tr(
    fst: *mut CFst,
    state: CStateId,
    tr: *const CTr,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let tr = unsafe { <CTr as ffi_convert::RawBorrow<CTr>>::raw_borrow(tr)? }.as_rust()?;
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        vec_fst.add_tr(state, tr)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_del_final_weight(fst: *mut CFst, state: CStateId) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);
        vec_fst.delete_final_weight(state)?;

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_from_path(
    ptr: *mut *const CFst,
    path: *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let fst = Box::new(VectorFst::<TropicalWeight>::read(path)?);
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_write_file(
    fst: *const CFst,
    path: *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        vec_fst.write(path)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_relabel_tables(
    fst: *mut CFst,
    old_isymbols: *const CSymbolTable,
    new_isymbols: *const CSymbolTable,
    attach_new_isymbols: libc::size_t,
    old_osymbols: *const CSymbolTable,
    new_osymbols: *const CSymbolTable,
    attach_new_osymbols: libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst);
        let vec_fst: &mut VectorFst<_> = as_mut_fst!(VectorFst<TropicalWeight>, fst);

        let old_isymbols = get_symt(old_isymbols)?;
        let new_isymbols =
            get_symt(new_isymbols)?.ok_or_else(|| format_err!("New isymbols ptr is null"))?;
        let old_osymbols = get_symt(old_osymbols)?;
        let new_osymbols =
            get_symt(new_osymbols)?.ok_or_else(|| format_err!("New osymbols ptr is null"))?;

        let attach_new_isymbols = attach_new_isymbols > 0;
        let attach_new_osymbols = attach_new_osymbols > 0;

        vec_fst.relabel_tables(
            old_isymbols,
            new_isymbols,
            attach_new_isymbols,
            old_osymbols,
            new_osymbols,
            attach_new_osymbols,
        )?;

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe fn vec_fst_draw(
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
        let vec_fst = as_mut_fst!(VectorFst<TropicalWeight>, fst);

        if !isyms.is_null() {
            let isymt = get!(CSymbolTable, isyms);
            vec_fst.fst_set_input_symbols(isymt.clone());
        }

        if !osyms.is_null() {
            let osymt = get!(CSymbolTable, osyms);
            vec_fst.fst_set_output_symbols(osymt.clone());
        }

        let drawing_config = DrawingConfig {
            vertical: vertical > 0,
            size: if width >= 0.0 && height >= 0.0 {
                Some((width, height))
            } else {
                None
            },
            title: unsafe { CStr::from_ptr(title).as_rust()? },
            portrait: portrait > 0,
            ranksep: if ranksep >= 0.0 { Some(ranksep) } else { None },
            nodesep: if nodesep >= 0.0 { Some(nodesep) } else { None },
            fontsize: fontsize as u32,
            acceptor: acceptor > 0,
            show_weight_one: show_weight_one > 0,
            print_weight: print_weight > 0,
        };

        vec_fst.draw(unsafe { CStr::from_ptr(fname).as_rust()? }, &drawing_config)?;

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_num_states(
    fst: *const CFst,
    num_states: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        let res = vec_fst.num_states();
        unsafe { *num_states = res };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn vec_fst_equals(
    fst: *const CFst,
    other_fst: *const CFst,
    is_equal: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let other_fst = get!(CFst, other_fst);
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        let other_vec_fst = as_fst!(VectorFst<TropicalWeight>, other_fst);
        let res = vec_fst.eq(other_vec_fst);
        unsafe { *is_equal = res as usize }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn vec_fst_copy(
    fst_ptr: *const CFst,
    clone_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        let clone = vec_fst.clone();
        unsafe { *clone_ptr = CFst(Box::new(clone)).into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn vec_fst_display(
    fst_ptr: *const CFst,
    s: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        let res = format!("{}", vec_fst);
        unsafe { *s = CString::c_repr_of(res)?.into_raw_pointer() as *const libc::c_char };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn vec_fst_to_bytes(
    fst_ptr: *const CFst,
    output_bytes: *mut *const CArray<u8>,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<_> = as_fst!(VectorFst<TropicalWeight>, fst);

        let mut bytes = vec![];
        vec_fst.store(&mut bytes)?;

        let c_bytes = CArray::<u8>::c_repr_of(bytes)?;
        let raw_pointer = c_bytes.into_raw_pointer();
        unsafe { *output_bytes = raw_pointer };

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn vec_fst_from_bytes(
    bytes: *const CArray<u8>,
    ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let bytes = unsafe { CArray::raw_borrow(bytes)? };
        let bytes = bytes.as_rust()?;
        let fst = VectorFst::load(bytes.as_slice())?;
        let raw_pointer = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}
