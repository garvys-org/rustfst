use anyhow::{anyhow, Result};

use crate::fst::CFst;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::*;
use rustfst::algorithms::compose::ComposeFst;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[no_mangle]
pub fn compose_fst_new(
    ptr: *mut *const CFst,
    fst_1: *const CFst,
    fst_2: *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_1 = get!(CFst, fst_1);
        let vec_fst1: &VectorFst<TropicalWeight> = fst_1
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst_2 = get!(CFst, fst_2);
        let vec_fst2: &VectorFst<TropicalWeight> = fst_2
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let fst = ComposeFst::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            _,
            _,
            _,
            _,
            _,
        >::new_auto(vec_fst1, vec_fst2)?;
        let raw_pointer = CFst(Box::new(fst)).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}


#[no_mangle]
pub fn compose_fst_compute(
    ptr: *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get!(CFst, ptr);
        let vec_fst: &ComposeFst<TropicalWeight, VectorFst<TropicalWeight>, VectorFst<TropicalWeight>,_,_,_,_,_,_> = fst
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let res: VectorFst<TropicalWeight> = vec_fst.compute()?;

        Ok(())
    })
}