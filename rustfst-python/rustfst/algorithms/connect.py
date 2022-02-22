from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def connect(fst: VectorFst):
    """
    connect(fst)
    connect an fst
    :param fst: Fst
    :return: Fst
    """

    connectd_fst = ctypes.c_void_p()
    ret_code = lib.fst_connect(fst.ptr, ctypes.byref(connectd_fst))
    err_msg = "Error during connect"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=connectd_fst)
