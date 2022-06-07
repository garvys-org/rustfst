from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def tr_sort(fst: VectorFst, ilabel_cmp: bool):
    """
    tr_sort(fst)
    sort fst trs according to their ilabel or olabel
    :param fst: Fst
    :param ilabel_cmp: bool
    """

    ret_code = lib.fst_tr_sort(fst.ptr, ctypes.c_bool(ilabel_cmp))
    err_msg = "Error during tr_sort"
    check_ffi_error(ret_code, err_msg)
