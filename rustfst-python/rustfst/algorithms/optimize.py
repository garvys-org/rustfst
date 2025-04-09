from __future__ import annotations

import ctypes

from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def optimize(fst: VectorFst) -> VectorFst:
    """
    Optimize an fst in-place

    Args:
        fst: Fst to optimize.
    """
    ret_code = lib.fst_optimize(fst.ptr)
    err_msg = "Error during optimize"
    check_ffi_error(ret_code, err_msg)
    return fst


def optimize_in_log(fst: VectorFst) -> VectorFst:
    """
    Optimize an fst in-place in the log semiring.

    Args:
        fst: Fst to optimize.
    """
    ret_code = lib.fst_optimize_in_log(ctypes.byref(fst.ptr))
    err_msg = "Error during optimize_in_log"
    check_ffi_error(ret_code, err_msg)
    return fst
