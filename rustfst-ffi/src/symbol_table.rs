use crate::{get, get_mut, wrap, CStateId, RUSTFST_FFI_RESULT};
use ffi_convert::*;
use std::ffi::{CStr, CString};
use std::sync::Arc;

use anyhow::{anyhow, format_err};
use rustfst::SymbolTable;

#[derive(RawPointerConverter)]
pub struct CSymbolTable(pub(crate) Arc<SymbolTable>);

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_new(new_struct: *mut *const CSymbolTable) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let table = Arc::new(SymbolTable::new());
        let raw_ptr = CSymbolTable(table).into_raw_pointer();
        unsafe { *new_struct = raw_ptr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_add_symbol(
    symt: *mut CSymbolTable,
    symbol: *const libc::c_char,
    integer_key: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get_mut!(CSymbolTable, symt);
        let symbol: String = unsafe { CStr::from_ptr(symbol) }.as_rust()?;
        let res = Arc::get_mut(symt)
            .ok_or_else(|| anyhow!("Could not get a mutable reference to the symbol table"))?
            .add_symbol(&symbol);
        unsafe { *integer_key = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_add_table(
    symt: *mut CSymbolTable,
    other_symt: *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get_mut!(CSymbolTable, symt);
        let other_symt = get!(CSymbolTable, other_symt);
        Arc::get_mut(symt)
            .ok_or_else(|| anyhow!("Could not get a mutable reference to the symbol table"))?
            .add_table(other_symt);
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_find_index(
    symt: *const CSymbolTable,
    key: CStateId,
    symbol: *mut *const libc::c_char,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let found_symbol = symt
            .get_symbol(key)
            .ok_or_else(|| format_err!("No symbol found at index:{}", key as i32))?;
        unsafe {
            *symbol = CString::c_repr_of(found_symbol.to_string())?.into_raw_pointer()
                as *const libc::c_char
        };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_find_symbol(
    symt: *const CSymbolTable,
    symbol: *const libc::c_char,
    key: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let symbol = unsafe { CStr::from_ptr(symbol) }.as_rust()?;
        let res = symt
            .get_label(&symbol)
            .ok_or_else(|| format_err!("No symbol `{}` found", symbol))?;
        unsafe { *key = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_from_path(
    table_ptr: *mut *const CSymbolTable,
    path_ptr: *const libc::c_char,
    binary: *const libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let path = unsafe { CStr::from_ptr(path_ptr) }.as_rust()?;
        let binary = binary as i32 != 0;
        let symb = if binary {
            SymbolTable::read(&path)?
        } else {
            SymbolTable::read_text(&path)?
        };
        let raw_ptr = CSymbolTable(Arc::new(symb)).into_raw_pointer();
        unsafe { *table_ptr = raw_ptr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_write_file(
    symt: *const CSymbolTable,
    path_ptr: *const libc::c_char,
    binary: *const libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let path = unsafe { CStr::from_ptr(path_ptr) }.as_rust()?;
        let binary = binary as i32 != 0;
        if binary {
            symt.write(&path)?
        } else {
            symt.write_text(&path)?
        };

        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_member_index(
    symt: *const CSymbolTable,
    key: CStateId,
    is_present: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let res = symt.contains_label(key);
        unsafe { *is_present = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_member_symbol(
    symt: *const CSymbolTable,
    symbol: *const libc::c_char,
    is_present: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let symbol = unsafe { CStr::from_ptr(symbol) }.as_rust()?;
        let res = symt.contains_symbol(symbol);
        unsafe { *is_present = res as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_num_symbols(
    symt: *const CSymbolTable,
    num_symbols: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        unsafe { *num_symbols = symt.len() as libc::size_t };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_copy(
    symt: *const CSymbolTable,
    cloned_symt: *mut *const CSymbolTable,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let clone = Arc::new(SymbolTable::clone(symt));
        let raw_ptr = CSymbolTable(clone).into_raw_pointer();
        unsafe { *cloned_symt = raw_ptr };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe fn symt_equals(
    symt: *const CSymbolTable,
    other_symt: *const CSymbolTable,
    is_equal: *mut libc::size_t,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let symt = get!(CSymbolTable, symt);
        let other_symt = get!(CSymbolTable, other_symt);
        let res = symt.eq(other_symt);
        unsafe { *is_equal = res as usize }
        Ok(())
    })
}

/// drop impl
/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn symt_destroy(symt_ptr: *mut CSymbolTable) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        if symt_ptr.is_null() {
            return Ok(());
        }

        drop(unsafe { Box::from_raw(symt_ptr) });
        Ok(())
    })
}
