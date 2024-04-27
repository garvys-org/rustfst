use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::{shortest_path, shortest_path_with_config, ShortestPathConfig};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[derive(AsRust, CReprOf, CDrop, RawPointerConverter)]
#[target_type(ShortestPathConfig)]
pub struct CShortestPathConfig {
    delta: f32,
    nshortest: usize,
    unique: bool,
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_shortest_path_config_new(
    delta: libc::c_float,
    nshortest: libc::size_t,
    unique: bool,
    ptr: *mut *const CShortestPathConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let config = CShortestPathConfig {
            delta,
            nshortest,
            unique,
        };
        unsafe { *ptr = config.into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_shortest_path(
    ptr: *const CFst,
    res_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let res: VectorFst<TropicalWeight> = shortest_path(vec_fst)?;
        unsafe { *res_fst = CFst(Box::new(res)).into_raw_pointer() };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_shortest_path_with_config(
    ptr: *const CFst,
    config: *const CShortestPathConfig,
    res_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let config = unsafe {
            <CShortestPathConfig as ffi_convert::RawBorrow<CShortestPathConfig>>::raw_borrow(
                config,
            )?
        };
        let res: VectorFst<TropicalWeight> = shortest_path_with_config(vec_fst, config.as_rust()?)?;
        unsafe { *res_fst = CFst(Box::new(res)).into_raw_pointer() };
        Ok(())
    })
}
