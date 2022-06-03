from __future__ import annotations
import ctypes
from typing import Optional

from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst
from enum import Enum

KDELTA = 1.0 / 1024.0


class DeterminizeType(Enum):
    """
    Enumeration defining the type of the determinization to perform.
    """

    DETERMINIZE_FUNCTIONAL = 0
    """
    Input transducer is known to be functional (or error).
    """
    DETERMINIZE_NON_FUNCTIONAL = 1
    """
    Input transducer is NOT known to be functional.
    """
    DETERMINIZE_DISAMBIGUATE = 2
    """
    Input transducer is not known to be functional but only keep the min of
    of ambiguous outputs.
    """


class DeterminizeConfig:
    """
    Struct containing the parameters controlling the determinization algorithm.
    """

    def __init__(self, det_type: DeterminizeType, delta: Optional[float] = None):
        """
        Creates the configuration object.
        Args:
            det_type: Type of determinization to perform.
            delta:
        """
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
    Make an Fst deterministic
    Args:
        fst: The Fst to make deterministic.
    Returns:
        The resulting Fst.
    """
    det_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_determinize(fst.ptr, ctypes.byref(det_fst))
    err_msg = "Error during determinization"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=det_fst)


def determinize_with_config(fst: VectorFst, config: DeterminizeConfig) -> VectorFst:
    """
    Make an Fst deterministic
    Args:
        fst: The Fst to make deterministic.
        config: Configuration of the determinization algorithm to use.
    Returns:
        The resulting Fst.
    """
    det_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_determinize_with_config(
        fst.ptr, config.ptr, ctypes.byref(det_fst)
    )
    err_msg = "Error during determinization"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=det_fst)
