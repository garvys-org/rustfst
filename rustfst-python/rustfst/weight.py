import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)


def weight_one() -> float:
    """
    Compute One() in the Tropical Semiring.
    Returns:
        Float value corresponding to One() in the Tropical Semiring.
    """
    weight = ctypes.c_float()
    ret_code = lib.fst_weight_one(ctypes.byref(weight))
    err_msg = "weight_one failed"
    check_ffi_error(ret_code, err_msg)
    return float(weight.value)


def weight_zero() -> float:
    """
    Compute Zero() in the Tropical Semiring.
    Returns:
        Float value corresponding to Zero() in the Tropical Semiring.
    """
    weight = ctypes.c_float()
    ret_code = lib.fst_weight_zero(ctypes.byref(weight))
    err_msg = "weight_zero failed"
    check_ffi_error(ret_code, err_msg)
    return float(weight.value)
