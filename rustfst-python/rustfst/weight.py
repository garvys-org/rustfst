import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)


def weight_one():
    weight = ctypes.c_float()
    ret_code = lib.fst_weight_one(ctypes.byref(weight))
    err_msg = "weight_one failed"
    check_ffi_error(ret_code, err_msg)
    return float(weight.value)


def weight_zero():
    weight = ctypes.c_float()
    ret_code = lib.fst_weight_zero(ctypes.byref(weight))
    err_msg = "weight_zero failed"
    check_ffi_error(ret_code, err_msg)
    return float(weight.value)
