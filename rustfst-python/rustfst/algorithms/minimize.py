from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst

KSHORTESTDELTA = 1e-6


class MinimizeConfig:
    def __init__(self, delta=None, allow_nondet=False):
        if delta is None:
            delta = KSHORTESTDELTA
        config = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_minimize_config_new(
            ctypes.c_float(delta),
            ctypes.c_bool(allow_nondet),
            ctypes.byref(config),
        )
        err_msg = "Error creating MinimizeConfig"
        check_ffi_error(ret_code, err_msg)
        self.ptr = config


def minimize(fst: VectorFst) -> VectorFst:
    """
    minimize(fst)
    Minimize a FST in place
    :param fst: Fst
    :return: Fst
    """
    ret_code = lib.fst_minimize(fst.ptr)
    err_msg = "Error while minimizing FST"
    check_ffi_error(ret_code, err_msg)

    return fst


def minimize_with_config(fst: VectorFst, config: MinimizeConfig) -> VectorFst:
    """
    minimize(fst, config)
    Minimize a FST in place
    :param fst: Fst
    :param config: MinimizeConfig
    :return: Fst
    """
    ret_code = lib.fst_minimize_with_config(fst.ptr, config.ptr)
    err_msg = "Error while minimizing FST"
    check_ffi_error(ret_code, err_msg)

    return fst
