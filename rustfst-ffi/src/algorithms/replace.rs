use anyhow::{anyhow, Result};

use crate::fst::CFst;
use crate::CLabel;
use crate::{get, wrap, RUSTFST_FFI_RESULT};

use ffi_convert::RawPointerConverter;
use rustfst::algorithms::replace::replace;
use rustfst::prelude::{Label, TropicalWeight, VectorFst};

pub struct CLabelFstPair {
    pub label: CLabel,
    pub fst: *const CFst,
}

#[no_mangle]
pub extern "C" fn fst_replace(
    fst_list_ptr: *const CLabelFstPair,
    fst_list_ptr_len: libc::size_t,
    root: CLabel,
    epsilon_on_replace: bool,
    replaced_fst: *mut *const CFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let label_fst_pairs =
            unsafe { std::slice::from_raw_parts(fst_list_ptr, fst_list_ptr_len as usize) };
        let fst_list = label_fst_pairs
            .into_iter()
            .map(|pair| -> Result<(CLabel, &VectorFst<TropicalWeight>)> {
                let fst_ptr = pair.fst;
                let fst = get!(CFst, fst_ptr);
                let vec_fst: &VectorFst<TropicalWeight> = fst
                    .downcast_ref()
                    .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;
                Ok((pair.label as Label, vec_fst))
            })
            .collect::<Result<Vec<(CLabel, &VectorFst<TropicalWeight>)>>>()?;

        let res_fst: VectorFst<TropicalWeight> = replace::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            _,
            _,
        >(fst_list, root, epsilon_on_replace)?;
        unsafe { *replaced_fst = CFst(Box::new(res_fst)).into_raw_pointer() };
        Ok(())
    })
}