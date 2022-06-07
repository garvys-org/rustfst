from __future__ import annotations

from typing import List

from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def concat(fst: VectorFst, other_fst: VectorFst) -> VectorFst:
    """
    Compute the concatenation of two Fsts.
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


def concat_list(fsts: List[VectorFst]) -> VectorFst:
    """
    Compute the concatenation of a list of Fsts.
    Args:
        fsts: List of Fsts to concatenated

    Returns:
        The resulting concatenated Fst.
    """
    if not fsts:
        raise ValueError("fsts must be at least of len 1")
    fsts = [f.copy() for f in fsts]
    concatenated = fsts[0]
    for f in fsts[1:]:
        concatenated = concatenated.concat(f)
    return concatenated
