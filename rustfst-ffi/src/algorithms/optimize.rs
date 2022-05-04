use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};

use rustfst::algorithms::optimize;
use rustfst::fst_impls::VectorFst;
use rustfst::semirings::TropicalWeight;

#[no_mangle]
pub extern "C" fn fst_optimize(ptr: *mut CFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
        optimize(vec_fst)?;
        Ok(())
    })
}
