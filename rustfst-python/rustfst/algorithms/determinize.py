from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst
from enum import Enum

KDELTA = 1.0 / 1024.0


class DeterminizeType(Enum):
    DETERMINIZE_FUNCTIONAL = 0
    DETERMINIZE_NON_FUNCTIONAL = 1
    DETERMINIZE_DISAMBIGUATE = 2


class DeterminizeConfig:
    def __init__(self, det_type: DeterminizeType, delta=None):
        if delta is None:
            delta = KDELTA

        config = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_determinize_config_new(
            ctypes.c_float(delta),
            ctypes.c_size_t(det_type.value),
            ctypes.byref(config),
        )
        err_msg = "Error creating DeterminizeConfig"
        check_ffi_error(ret_code, err_msg)
        self.ptr = config


def determinize(fst: VectorFst) -> VectorFst:
    """
    determinize(fst)
    make an fst deterministic
    :param fst: Fst
    :return: Fst
    """
    det_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_determinize(fst.ptr, ctypes.byref(det_fst))
    err_msg = "Error during determinization"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=det_fst)


def determinize_with_config(fst: VectorFst, config: DeterminizeConfig) -> VectorFst:
    """
    determinize(fst)
    make an fst deterministic
    :param fst: Fst
    :param config: DeterminizeConfig
    :return: Fst
    """
    det_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_determinize_with_config(
        fst.ptr, config.ptr, ctypes.byref(det_fst)
    )
    err_msg = "Error during determinization"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=det_fst)
