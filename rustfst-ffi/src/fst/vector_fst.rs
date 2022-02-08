use super::CVecFst;
use super::*;

//#[no_mangle]
//pub fn vec_fst_start(fst: *const CVecFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
//    fst_start(fst, state)
//}
//
//#[no_mangle]
//pub fn vec_fst_final_weight(
//    fst: *const CVecFst,
//    state_id: CStateId,
//    final_weight: *mut libc::c_float,
//) -> RUSTFST_FFI_RESULT {
//    fst_final_weight(fst, state_id, final_weight)
//}
//
//#[no_mangle]
//pub fn vec_fst_num_trs(
//    fst: *const CVecFst,
//    state: CStateId,
//    num_trs: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_num_trs(fst, state, num_trs)
//}
//
//#[no_mangle]
//pub fn vec_fst_is_final(
//    fst: *const CVecFst,
//    state: CStateId,
//    is_final: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_is_final(fst, state, is_final)
//}
//
//#[no_mangle]
//pub fn vec_fst_is_start(
//    fst: *const CVecFst,
//    state: CStateId,
//    is_start: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_is_start(fst, state, is_start)
//}
//
//#[no_mangle]
//pub fn vec_fst_input_symbols(
//    fst: *const CVecFst,
//    input_symt: *mut CSymbolTable,
//) -> RUSTFST_FFI_RESULT {
//    fst_input_symbols(fst, input_symt)
//}
//
//#[no_mangle]
//pub fn vec_fst_output_symbols(
//    fst: *const CVecFst,
//    output_symt: *mut CSymbolTable,
//) -> RUSTFST_FFI_RESULT {
//    fst_output_symbols(fst, output_symt)
//}
//
//#[no_mangle]
//pub fn vec_fst_set_input_symbols(
//    fst: *mut CVecFst,
//    symt: *const CSymbolTable,
//) -> RUSTFST_FFI_RESULT {
//    fst_set_input_symbols(fst, symt)
//}
//
//#[no_mangle]
//pub fn vec_fst_set_output_symbols(
//    fst: *mut CVecFst,
//    symt: *const CSymbolTable,
//) -> RUSTFST_FFI_RESULT {
//    fst_set_output_symbols(fst, symt)
//}
//
//#[no_mangle]
//pub fn vec_fst_unset_input_symbols(fst: *mut CVecFst) -> RUSTFST_FFI_RESULT {
//    fst_unset_input_symbols(fst)
//}
//
//#[no_mangle]
//pub fn vec_fst_unset_output_symbols(fst: *mut CVecFst) -> RUSTFST_FFI_RESULT {
//    fst_unset_output_symbols(fst)
//}

#[no_mangle]
pub fn vec_fst_new(ptr: *mut *const CVecFst) -> RUSTFST_FFI_RESULT {
    wrap(|| {
        let fst = Box::new(VectorFst::new());
        let raw_pointer = CVecFst(fst).into_raw_pointer();
        unsafe { *ptr = raw_pointer };
        Ok(())
    })
}

//#[no_mangle]
//pub fn vec_fst_set_start(fst: *mut CVecFst, state: CStateId) -> RUSTFST_FFI_RESULT {
//    fst_set_start(fst, state)
//}
//
//#[no_mangle]
//pub fn vec_fst_set_final(
//    fst: *mut CVecFst,
//    state: CStateId,
//    weight: libc::c_float,
//) -> RUSTFST_FFI_RESULT {
//    fst_set_final(fst, state, weight)
//}
//
//#[no_mangle]
//pub fn vec_fst_add_state(fst: *mut CVecFst, state: *mut CStateId) -> RUSTFST_FFI_RESULT {
//    fst_add_state(fst, state)
//}
//
//#[no_mangle]
//pub fn vec_fst_delete_states(fst: *mut CVecFst) -> RUSTFST_FFI_RESULT {
//    fst_delete_states(fst)
//}
//
//#[no_mangle]
//pub fn vec_fst_add_tr(fst: *mut CVecFst, state: CStateId, tr: *const CTr) -> RUSTFST_FFI_RESULT {
//    fst_add_tr(fst, state, tr)
//}
//
//#[no_mangle]
//pub fn vec_fst_from_path(
//    ptr: *mut *const CVecFst,
//    path: *const libc::c_char,
//) -> RUSTFST_FFI_RESULT {
//    fst_from_path(ptr, path)
//}
//
//#[no_mangle]
//pub fn vec_fst_write_file(fst: *const CVecFst, path: *const libc::c_char) -> RUSTFST_FFI_RESULT {
//    fst_write_file(fst, path)
//}
//
//#[no_mangle]
//pub fn vec_fst_draw(
//    fst_ptr: *mut CVecFst,
//    isyms: *const CSymbolTable,
//    osyms: *const CSymbolTable,
//    fname: *const libc::c_char,
//    title: *const libc::c_char,
//    acceptor: libc::size_t,
//    width: libc::c_float,
//    height: libc::c_float,
//    portrait: libc::size_t,
//    vertical: libc::size_t,
//    ranksep: libc::c_float,
//    nodesep: libc::c_float,
//    fontsize: libc::size_t,
//    show_weight_one: libc::size_t,
//    print_weight: libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_draw(
//        fst_ptr,
//        isyms,
//        osyms,
//        fname,
//        title,
//        acceptor,
//        width,
//        height,
//        portrait,
//        vertical,
//        ranksep,
//        nodesep,
//        fontsize,
//        show_weight_one,
//        print_weight,
//    )
//}
//
//#[no_mangle]
//pub fn vec_fst_num_states(
//    fst: *const CVecFst,
//    num_states: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_num_states(fst, num_states)
//}
//
//#[no_mangle]
//pub fn vec_fst_equals(
//    fst: *const CVecFst,
//    other_fst: *const CVecFst,
//    is_equal: *mut libc::size_t,
//) -> RUSTFST_FFI_RESULT {
//    fst_equals(fst, other_fst, is_equal)
//}
//
///// drop impl
//#[no_mangle]
//pub fn vec_fst_destroy(fst_ptr: *mut CVecFst) -> RUSTFST_FFI_RESULT {
//    fst_destroy(fst_ptr)
//}
