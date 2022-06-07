from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def top_sort(fst: VectorFst) -> VectorFst:
    """
    This operation topologically sorts its input. When sorted, all transitions are from lower to higher state IDs.

    Examples :

    - Input

    ![topsort_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_in.svg?sanitize=true)

    - Output

    ![topsort_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_out.svg?sanitize=true)

    Args:
        fst: Fst to top_sort.
    Returns:
        Equivalent top sorted Fst. Modification also happens in-place.
    """

    top_sorted_fst = ctypes.c_void_p()
    ret_code = lib.fst_top_sort(fst.ptr, ctypes.byref(top_sorted_fst))
    err_msg = "Error during top_sort"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=top_sorted_fst)
