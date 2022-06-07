from __future__ import annotations
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def tr_unique(fst: VectorFst):
    """
    Keep a single instance of trs leaving the same state, going to the same state and
    with the same input labels, output labels and weight.
    Args:
        fst: Fst to modify
    """

    ret_code = lib.fst_tr_unique(fst.ptr)
    err_msg = "Error during tr_unique"
    check_ffi_error(ret_code, err_msg)
