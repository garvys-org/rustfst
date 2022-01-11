from ctypes import byref, c_void_p
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.tr import Tr
from typing import List

class Trs:
    """Structure representing list of transitions."""

    def __init__(self, ptr=None):
        if ptr is None:
            self._ptr = c_void_p()
            exit_code = lib.trs_vec_new(byref(self._ptr))
            err_msg = "Something went wrong when creating the Trs struct"
            check_ffi_error(exit_code, err_msg)
        else:
            self._ptr = ptr

    def push(self, tr: Tr):
        exit_code = lib.trs_vec_push(self._ptr, tr._ptr)
        err_msg = "Something went wrong when adding new transition"
        check_ffi_error(exit_code, err_msg)

    def remove(self, index):
        removed_tr = c_void_p()
        exit_code = lib.trs_vec_remove(self._ptr, index, byref(removed_tr))
        err_msg = "Something went wrong when removing transition at index: " + str(
            index
        )
        check_ffi_error(exit_code, err_msg)

    def shallow_clone(self):
        new_trs_ptr = c_void_p()
        exit_code = lib.trs_vec_shallow_clone(self._ptr, new_trs_ptr)
        err_msg = "Something went wrong when cloning Trs"
        check_ffi_error(exit_code, err_msg)

        return Trs(new_trs_ptr)

    def __del__(self):
        lib.trs_vec_delete(self._ptr)
