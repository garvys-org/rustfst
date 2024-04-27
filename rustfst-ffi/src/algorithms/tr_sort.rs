use anyhow::anyhow;

use crate::fst::CFst;
use crate::{get_mut, wrap, RUSTFST_FFI_RESULT};

use rustfst::algorithms::tr_sort;
use rustfst::fst_impls::VectorFst;
use rustfst::prelude::{ILabelCompare, OLabelCompare};
use rustfst::semirings::TropicalWeight;

/// # Safety
///
/// The pointers should be valid.
#[no_mangle]
pub unsafe extern "C" fn fst_tr_sort(ptr: *mut CFst, ilabel_comp: bool) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = get_mut!(CFst, ptr);
        let vec_fst: &mut VectorFst<TropicalWeight> = fst
            .downcast_mut()
            .ok_or_else(|| anyhow!("Could not downcast to vector FST"))?;

        if ilabel_comp {
            tr_sort(vec_fst, ILabelCompare {});
        } else {
            tr_sort(vec_fst, OLabelCompare {});
        };

        Ok(())
    })
}
