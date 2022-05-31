use super::*;
use anyhow::anyhow;
use rustfst::DrawingConfig;
use std::ffi::CString;

#[no_mangle]
pub fn const_fst_from_path(ptr: *mut *const CFst, path: *const libc::c_char) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let fst = Box::new(ConstFst::<TropicalWeight>::read(&path)?);
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

#[no_mangle]
pub fn const_fst_write_file(fst: *const CFst, path: *const libc::c_char) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst);
        let path = unsafe { CStr::from_ptr(path) }.as_rust()?;
        let const_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        const_fst.write(&path)?;
        Ok(())
    })
}

#[no_mangle]
pub fn const_fst_equals(
    fst: *const CFst,
    other_fst: *const CFst,
    is_equal: *mut libc::size_t,
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

#[no_mangle]
pub extern "C" fn const_fst_copy(
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

#[no_mangle]
pub fn const_fst_draw(
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
            vertical: if vertical > 0 { true } else { false },
            size: if width >= 0.0 && height >= 0.0 {
                Some((width, height))
            } else {
                None
            },
            title: unsafe { CStr::from_ptr(title).as_rust()? },
            portrait: if portrait > 0 { true } else { false },
            ranksep: if ranksep >= 0.0 { Some(ranksep) } else { None },
            nodesep: if nodesep >= 0.0 { Some(nodesep) } else { None },
            fontsize: fontsize as u32,
            acceptor: if acceptor > 0 { true } else { false },
            show_weight_one: if show_weight_one > 0 { true } else { false },
            print_weight: if print_weight > 0 { true } else { false },
        };

        const_fst.draw(unsafe { CStr::from_ptr(fname).as_rust()? }, &drawing_config)?;

        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn const_fst_display(
    fst_ptr: *const CFst,
    s: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, fst_ptr);
        let vec_fst = as_fst!(ConstFst<TropicalWeight>, fst);
        let res = format!("{}", vec_fst);
        unsafe { *s = CString::c_repr_of(res)?.into_raw_pointer() as *const libc::c_char };
        Ok(())
    })
}
