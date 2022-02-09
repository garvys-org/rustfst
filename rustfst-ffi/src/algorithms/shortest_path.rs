use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::{shortest_path, shortest_path_with_config, ShortestPathConfig};
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[derive(AsRust, CReprOf, CDrop)]
#[target_type(ShortestPathConfig)]
pub struct CShortestPathConfig {
    delta: f32,
    nshortest: usize,
    unique: bool,
}

#[no_mangle]
pub extern "C" fn fst_shortest_path(ptr: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let res_fst: VectorFst<TropicalWeight> = shortest_path(vec_fst)?;
        unsafe { *ptr = CFst(Box::new(res_fst)).into_raw_pointer() };
        Ok(())
    })
}

#[no_mangle]
pub extern "C" fn fst_shortest_path_with_config(
    ptr: *mut *const CFst,
    config: *const CShortestPathConfig,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_ptr = unsafe { *ptr };
        let fst = get!(CFst, fst_ptr);
        let vec_fst: &VectorFst<TropicalWeight> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let config = unsafe {
            <CShortestPathConfig as ffi_convert::RawBorrow<CShortestPathConfig>>::raw_borrow(
                config,
            )?
        };
        let res_fst: VectorFst<TropicalWeight> =
            shortest_path_with_config(vec_fst, config.as_rust()?)?;
        unsafe { *ptr = CFst(Box::new(res_fst)).into_raw_pointer() };
        Ok(())
    })
}
