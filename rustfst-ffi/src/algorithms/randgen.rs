use anyhow::anyhow;
use ffi_convert::RawPointerConverter;

use rustfst::algorithms::randgen::{randgen_with_config, RandGenConfig, UniformTrSelector};
use rustfst::prelude::{TropicalWeight, VectorFst};

use crate::fst::as_fst;
use crate::fst::CFst;
use crate::get;
use crate::{wrap, RUSTFST_FFI_RESULT};

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_randgen(
    ptr: *const CFst,
    npath: libc::size_t,
    seed: libc::size_t,
    max_length: libc::size_t,
    weight: bool,
    remove_total_weight: bool,
    res_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let ifst = get!(CFst, ptr);
        let ifst = as_fst!(VectorFst<TropicalWeight>, ifst);

        let config = RandGenConfig::new(UniformTrSelector::from_seed(seed as u64))
            .with_npath(npath)
            .with_weighted(weight)
            .with_max_length(max_length)
            .with_remove_total_weight(remove_total_weight);
        let res: VectorFst<_> = randgen_with_config(ifst, config)?;

        let fst_ptr = CFst(Box::new(res)).into_raw_pointer();
        unsafe { *res_fst = fst_ptr };
        Ok(())
    })
}
