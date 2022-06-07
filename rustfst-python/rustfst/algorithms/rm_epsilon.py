from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def rm_epsilon(fst: VectorFst):
    """
    rm_epsilon(fst)
    rm_epsilon an fst
    :param fst: Fst
    :return: Fst
    """

    rm_epsilon_fst = ctypes.c_void_p()
    ret_code = lib.fst_rm_epsilon(fst.ptr, ctypes.byref(rm_epsilon_fst))
    err_msg = "Error during rm_epsilon"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=rm_epsilon_fst)
