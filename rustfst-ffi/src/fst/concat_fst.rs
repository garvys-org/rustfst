use super::*;
use anyhow::anyhow;
use rustfst::algorithms::concat::ConcatFst;
use rustfst::prelude::{TropicalWeight, VectorFst};

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn concat_fst_new(
    ptr: *mut *const CFst,
    fst1: *const CFst,
    fst2: *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst1 = get!(CFst, fst1);
        let vec_fst1: &VectorFst<TropicalWeight> = fst1
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        let fst2 = get!(CFst, fst2);
        let vec_fst2: &VectorFst<TropicalWeight> = fst2
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst = Box::new(ConcatFst::<TropicalWeight, VectorFst<TropicalWeight>>::new(
            vec_fst1.clone(),
            vec_fst2.clone(),
        )?);
        let raw_pointer = CFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn concat_fst_compute(fst: *mut *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let c_fst = unsafe { *fst };
        let c_fst = get!(CFst, c_fst);
        let vec_fst: &ConcatFst<TropicalWeight, VectorFst<TropicalWeight>> =
            c_fst
                .downcast_ref()
                .ok_or_else(|| anyhow!("Could not downcast to concat FST"))?;
        let new_fst = Box::new(vec_fst.compute::<VectorFst<TropicalWeight>>()?);
        unsafe { *fst = CFst(new_fst).into_raw_pointer() }
        Ok(())
    })
}
