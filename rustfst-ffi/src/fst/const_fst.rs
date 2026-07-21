use super::*;
use anyhow::anyhow;
use rustfst::DrawingConfig;
use std::ffi::CString;

/// # Safety
///
/// The pointers should be valid.
#[cfg(feature = "std")]
#[no_mangle]
pub unsafe fn const_fst_from_path(
    ptr: *mut *const CFst,
    path: *const core::ffi::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let fst = Box::new(ConstFst::<TropicalWeight>::read(path)?);
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[cfg(feature = "std")]
#[no_mangle]
pub unsafe fn const_fst_write_file(
    fst: *const CFst,
    path: *const core::ffi::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let const_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        const_fst.write(path)?;
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn const_fst_equals(
    fst: *const CFst,
    other_fst: *const CFst,
    is_equal: *mut usize,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let other_fst = get!(CFst, other_fst);
        let const_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        let other_const_fst = as_fst!(ConstFst<TropicalWeight>, other_fst);
        let res = const_fst.eq(other_const_fst);
        unsafe { *is_equal = res as usize }
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn const_fst_copy(
    fst_ptr: *const CFst,
    clone_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let const_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        let clone = const_fst.clone();
        unsafe { *clone_ptr = CFst(Box::new(clone)).into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[allow(clippy::too_many_arguments)]
#[no_mangle]
pub unsafe fn const_fst_draw(
    fst_ptr: *mut CFst,
    isyms: *const CSymbolTable,
    osyms: *const CSymbolTable,
    fname: *const core::ffi::c_char,
    title: *const core::ffi::c_char,
    acceptor: usize,
    width: core::ffi::c_float,
    height: core::ffi::c_float,
    portrait: usize,
    vertical: usize,
    ranksep: core::ffi::c_float,
    nodesep: core::ffi::c_float,
    fontsize: usize,
    show_weight_one: usize,
    print_weight: usize,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, fst_ptr);
        let const_fst = as_mut_fst!(ConstFst<TropicalWeight>, fst);

        if !isyms.is_null() {
            let isymt = get!(CSymbolTable, isyms);
            const_fst.fst_set_input_symbols(isymt.clone());
        }

        if !osyms.is_null() {
            let osymt = get!(CSymbolTable, osyms);
            const_fst.fst_set_output_symbols(osymt.clone());
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

        const_fst.draw(unsafe { CStr::from_ptr(fname).as_rust()? }, &drawing_config)?;

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn const_fst_display(
    fst_ptr: *const CFst,
    s: *mut *const core::ffi::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let vec_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        let res = format!("{}", vec_fst);
        unsafe { *s = CString::c_repr_of(res)?.into_raw_pointer() as *const core::ffi::c_char };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn const_fst_from_vec_fst(
    vec_fst_prt: *const CFst,
    const_fst_ptr: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, vec_fst_prt);
        let vec_fst = as_fst!(VectorFst<TropicalWeight>, fst);
        let const_fst = ConstFst::from(vec_fst.clone());
        let raw_pointer = CFst(Box::new(const_fst)).into_raw_pointer();
        unsafe { *const_fst_ptr = raw_pointer };
        Ok(())
    })
}
