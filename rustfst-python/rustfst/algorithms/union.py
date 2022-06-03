from __future__ import annotations

from typing import List

from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def union(fst: VectorFst, other_fst: VectorFst) -> VectorFst:
    """
    Performs the union of two wFSTs. If A transduces string `x` to `y` with weight `a`
    and `B` transduces string `w` to `v` with weight `b`, then their union transduces `x` to `y`
    with weight `a` and `w` to `v` with weight `b`.

    Examples:
    - Input Fst 1:

    ![union_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_in_1.svg?sanitize=true)

    - Input Fst 2:

    ![union_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_in_2.svg?sanitize=true)

    - Union:

    ![union_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/union_out.svg?sanitize=true)

    Args:
        fst:
        other_fst:
    Returns:
         The resulting Fst.

    """

    ret_code = lib.fst_union(fst.ptr, other_fst.ptr)
    err_msg = "Error during union"
    check_ffi_error(ret_code, err_msg)

    return fst


def union_list(fsts: List[VectorFst]) -> VectorFst:
    """
    Computes the union of a list of Fsts.

    Args:
        fsts: The list of Fsts to produce the union.

    Returns:
        The resulting Fst.

    """
    if not fsts:
        raise ValueError("fsts must be at least of len 1")
    fsts = [f.copy() for f in fsts]
    res_fst = fsts[0]
    for f in fsts[1:]:
        res_fst = union(res_fst, f)
    return res_fst
