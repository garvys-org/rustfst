from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def tr_sort(fst: VectorFst, ilabel_cmp: bool) -> VectorFst:
    """Sort fst trs in place according to their input or output label.
    output label.

    This is often necessary for composition to work properly.  It
    corresponds to `ArcSort` in OpenFST.

    Args:
      fst: FST to be tr-sorted in-place.
      ilabel_cmp: Sort on input labels if `True`, output labels
                 if `False`.
    Returns:
      fst: Same FST that was modified in-place.
    """

    ret_code = lib.fst_tr_sort(fst.ptr, ctypes.c_bool(ilabel_cmp))
    err_msg = "Error during tr_sort"
    check_ffi_error(ret_code, err_msg)
    return fst
