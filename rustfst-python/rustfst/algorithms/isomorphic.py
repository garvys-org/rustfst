from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def isomorphic(fst: VectorFst, other_fst: VectorFst) -> bool:
    """
    Check if two Fsts are isomorphic.
    Args:
        fst: First Fst.
        other_fst: Second Fst.
    Returns:
        Whether both Fsts are equal.
    """

    is_isomorphic = ctypes.c_size_t()
    ret_code = lib.fst_isomorphic(fst.ptr, other_fst.ptr, ctypes.byref(is_isomorphic))
    err_msg = "Error checking isomorphic FSTs"
    check_ffi_error(ret_code, err_msg)

    return bool(is_isomorphic.value)
