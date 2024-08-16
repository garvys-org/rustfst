from __future__ import annotations
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def invert(fst: VectorFst) -> VectorFst:
    """
    Invert the transduction corresponding to an FST by exchanging the
    FST's input and output labels in-place.

    Args:
       fst: FST to be inverted.
    Returns:
       self
    """

    ret_code = lib.fst_invert(fst.ptr)
    err_msg = "Error during invert"
    check_ffi_error(ret_code, err_msg)

    return fst
