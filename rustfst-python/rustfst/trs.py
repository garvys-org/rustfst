from __future__ import annotations
from ctypes import byref, c_void_p, c_size_t, string_at
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.tr import Tr


class Trs:
    """Structure representing list of transitions."""

    def __init__(self, ptr=None) -> Trs:
        if ptr is None:
            self._ptr = c_void_p()
            exit_code = lib.trs_vec_new(byref(self._ptr))
            err_msg = "Something went wrong when creating the Trs struct"
            check_ffi_error(exit_code, err_msg)
        else:
            self._ptr = ptr

    def push(self, tr: Tr):
        exit_code = lib.trs_vec_push(self._ptr, tr.ptr)
        err_msg = "Something went wrong when adding new transition"
        check_ffi_error(exit_code, err_msg)

    def remove(self, index: int) -> Tr:
        removed_tr = c_void_p()
        exit_code = lib.trs_vec_remove(self._ptr, index, byref(removed_tr))
        err_msg = "Something went wrong when removing transition at index: " + str(
            index
        )
        check_ffi_error(exit_code, err_msg)
        return Tr(removed_tr)

    def len(self) -> int:
        num_trs = c_size_t()
        ret_code = lib.trs_vec_len(self._ptr, byref(num_trs))
        err_msg = "`len` failed"
        check_ffi_error(ret_code, err_msg)
        return int(num_trs.value)

    def shallow_clone(self) -> Trs:
        new_trs_ptr = c_void_p()
        exit_code = lib.trs_vec_shallow_clone(self._ptr, new_trs_ptr)
        err_msg = "Something went wrong when cloning Trs"
        check_ffi_error(exit_code, err_msg)

        return Trs(new_trs_ptr)

    def __repr__(self) -> str:
        string = c_void_p()
        exit_code = lib.trs_vec_display(self._ptr, byref(string))
        err_msg = "Something went wrong when displaying Trs"
        check_ffi_error(exit_code, err_msg)
        return string_at(string).decode("utf8")

    def __del__(self):
        lib.trs_vec_delete(self._ptr)
