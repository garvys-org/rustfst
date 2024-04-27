use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get, get_mut, wrap, RUSTFST_FFI_RESULT};

use rustfst::algorithms::concat::concat;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_concat(fst_1: *mut CFst, fst_2: *const CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst_1 = get_mut!(CFst, fst_1);
        let vec_fst1: &mut VectorFst<TropicalWeight> = fst_1
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        let fst_2 = get!(CFst, fst_2);
        let vec_fst2: &VectorFst<TropicalWeight> = fst_2
            .downcast_ref()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        concat(vec_fst1, vec_fst2)?;
        Ok(())
    })
}
