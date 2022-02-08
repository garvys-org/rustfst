use super::*;
use crate::trs::CTrs;
use rustfst::algorithms::concat::ConcatFst;
use rustfst::prelude::{TropicalWeight, VectorFst};

#[no_mangle]
pub extern "C" fn concat_fst_new(
    ptr: *mut *const CConcatFst,
    fst1: *const CVecFst,
    fst2: *const CVecFst,
) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst1 = get!(CVecFst, fst1);
        let fst2 = get!(CVecFst, fst2);
        let fst = Box::new(ConcatFst::<TropicalWeight, VectorFst<TropicalWeight>>::new(
            *fst1.clone(),
            *fst2.clone(),
        )?);
        let raw_pointer = CConcatFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

//#[no_mangle]
//pub extern "C" fn concat_fst_compute(fst: *mut *const CConcatFst) -> RUSTFST_FFI_RESULT {
//    wrap(|| {
//        let fst = get!(CConcatFst, fst);
//        let new_fst = fst.compute::<VectorFst<TropicalWeight>>()?;
//        unsafe { *fst = CFst(new_fst).into_raw_pointer() }
//        Ok(())
//    })
//}
//
//#[no_mangle]
//pub fn concat_fst_start(fst: *const CConcatFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
//    fst_start(fst, state)
//}
//
//#[no_mangle]
//pub fn concat_fst_final_weight(
//    fst: *const CConcatFst,
//    state_id: CStateId,
//    final_weight: *mut libc::c_float,
//) -> RUSTFST_FFI_RESULT {
//    fst_final_weight(fst, state_id, final_weight)
//}
//
//#[no_mangle]
//pub fn concat_fst_num_trs(
//    fst: *const CConcatFst,
//    state: CStateId,
//    num_trs: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_num_trs(fst, state, num_trs)
//}
//
//#[no_mangle]
//pub fn concat_fst_get_trs(
//    fst: *const CConcatFst,
//    state: CStateId,
//    trs: *mut *const CTrs,
//) -> RUSTFST_FFI_RESULT {
//    fst_get_trs(fst, state, trs)
//}
//
///// drop impl
//#[no_mangle]
//pub fn concat_fst_destroy(fst_ptr: *mut CConcatFst) -> RUSTFST_FFI_RESULT {
//    fst_destroy(fst_ptr)
//}
//
