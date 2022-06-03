from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def reverse(fst: VectorFst):
    """
    reverse(fst)
    reverse an fst
    :param fst: Fst
    :return: Fst
    """

    reversed_fst = ctypes.c_void_p()
    ret_code = lib.fst_reverse(fst.ptr, ctypes.byref(reversed_fst))
    err_msg = "Error during reverse"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=reversed_fst)
