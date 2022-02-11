from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst
from enum import Enum


class DeterminizeType(Enum):
    DETERMINIZE_FUNCTIONAL = 0
    DETERMINIZE_NON_FUNCTIONAL = 1
    DETERMINIZE_DISAMBIGUATE = 2


class DeterminizeConfig:
    def __init__(self, det_type=None, delta=None):
        if det_type and delta is None:
            self.ptr = det_type
        elif det_type and delta:
            config = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.fst_determinize_config_new(
                ctypes.c_float(delta),
                ctypes.c_size_t(det_type.value),
                ctypes.byref(config),
            )
            err_msg = "Error creating DeterminizeConfig"
            check_ffi_error(ret_code, err_msg)
            self.ptr = config
        else:
            raise ValueError("Could not create DeterminizeConfig")


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

    return det_fst


def determinize_with_config(fst: VectorFst, config: DeterminizeConfig) -> VectorFst:
    """
    determinize(fst)
    make an fst deterministic
    :param fst: Fst
    :param config: DeterminizeConfig
    :return: Fst
    """

    ret_code = lib.fst_determinize_with_config(fst.ptr, config.ptr)
    err_msg = "Error during determinization"
    check_ffi_error(ret_code, err_msg)

    return fst
