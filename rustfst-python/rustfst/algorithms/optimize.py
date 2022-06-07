from __future__ import annotations
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def optimize(fst: VectorFst):
    """
    Optimize an fst.
    Args:
        fst: Fst to optimize.
    """

    ret_code = lib.fst_optimize(fst.ptr)
    err_msg = "Error during optimize"
    check_ffi_error(ret_code, err_msg)
