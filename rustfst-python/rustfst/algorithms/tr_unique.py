from __future__ import annotations
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def tr_unique(fst: VectorFst):
    """
    tr_unique(fst)
    :param fst: Fst
    """

    ret_code = lib.fst_tr_unique(fst.ptr)
    err_msg = "Error during tr_unique"
    check_ffi_error(ret_code, err_msg)
