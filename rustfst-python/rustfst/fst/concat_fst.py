from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst import Fst


class ConcatFst(Fst):
    def __init__(self, ptr=None):
        if ptr:
            self._fst = ptr
        else:
            fst_ptr = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.vec_fst_new(ctypes.byref(fst_ptr))

            err_msg = "Something went wrong when creating the Fst struct"
            check_ffi_error(ret_code, err_msg)
            self._fst = fst_ptr

        super().__init__(self._fst)
