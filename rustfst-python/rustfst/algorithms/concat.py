from __future__ import annotations
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def concat(fst: VectorFst, other_fst: VectorFst) -> VectorFst:
    """
    Compute the concatenation of two FSTs.
    Args:
        fst: Left fst.
        other_fst: Right fst.
    Returns:
        Resulting fst.
    """

    ret_code = lib.fst_concat(fst.ptr, other_fst.ptr)
    err_msg = "Error during concat"
    check_ffi_error(ret_code, err_msg)

    return fst
