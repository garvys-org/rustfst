from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def reverse(fst: VectorFst) -> VectorFst:
    """
    Reverse an Fst, returning a new Fst which accepts
    the same language in reverse order.

    Not to be confused with `inverse`, which does something
    totally different!
    Args:
      fst: Fst to reverse
    Returns:
      Newly created, reversed Fst.
    """

    reversed_fst = ctypes.c_void_p()
    ret_code = lib.fst_reverse(fst.ptr, ctypes.byref(reversed_fst))
    err_msg = "Error during reverse"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=reversed_fst)
