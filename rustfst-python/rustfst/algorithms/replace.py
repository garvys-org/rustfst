from __future__ import annotations
from typing import List
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


class LabelFstPair(ctypes.Structure):
    _fields_ = [
        ("label", ctypes.c_size_t),
        ("fst", ctypes.POINTER(ctypes.c_void_p)),
    ]


# pub struct CLabelFstPair {
#    pub label: CLabel,
#    pub fst: *const CFst,
# }
#
##[no_mangle]
# pub extern "C" fn fst_replace(
#    fst_list_ptr: *const CLabelFstPair,
#    fst_list_ptr_len: libc::size_t,
#    root: CLabel,
#    epsilon_on_replace: bool,
#    replaced_fst: *mut *const CFst,


def replace(
    root_idx: int, fst_list: List[(int, VectorFst)], epsilon_on_replace: bool
) -> VectorFst:
    """
    replace(fst)
    constructively replaces arcs in an FST with other FST(s).
    :param root_idx: int
    :param fst_list: List[(int, VectorFst)]
    :param epsilon_on_replace: bool
    :return: Fst
    """

    pairs = [LabelFstPair(ctypes.c_size_t(label), fst.ptr) for (label, fst) in fst_list]
    pairs_array = (LabelFstPair * len(pairs))(*pairs)
    res_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_replace(
        ctypes.c_size_t(root_idx),
        ctypes.byref(pairs_array),
        ctypes.c_size_t(len(pairs)),
        ctypes.c_bool(epsilon_on_replace),
        ctypes.byref(res_fst),
    )
    err_msg = "Error performing replace"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=res_fst)
