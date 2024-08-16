from __future__ import annotations
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def rm_epsilon(fst: VectorFst) -> VectorFst:
    """
    Remove epsilon transitions in-place
    Args:
      fst: Fst to remove epsilons from
    Returns:
      fst: Same FST, modified in place
    """

    ret_code = lib.fst_rm_epsilon(fst.ptr)
    err_msg = "Error during rm_epsilon"
    check_ffi_error(ret_code, err_msg)

    return fst
