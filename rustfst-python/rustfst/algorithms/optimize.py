from __future__ import annotations
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def optimize(fst: VectorFst):
    """
    optimize(fst)
    omptimize an fst
    :param fst: Fst
    """

    ret_code = lib.fst_optimize(fst.ptr)
    err_msg = "Error during optimize"
    check_ffi_error(ret_code, err_msg)
