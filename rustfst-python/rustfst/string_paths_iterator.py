from __future__ import annotations
from typing import TYPE_CHECKING

from rustfst.ffi_utils import lib, check_ffi_error
from rustfst.string_path import StringPath
import ctypes

if TYPE_CHECKING:
    from rustfst.fst.vector_fst import VectorFst


class StringPathsIterator:
    def __init__(self, fst: VectorFst):
        self._fst = fst  # reference fst to prolong its lifetime (prevent early gc)

        iter_ptr = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.string_paths_iterator_new(fst.ptr, ctypes.byref(iter_ptr))
        error_msg = "Something went wrong when creating a StringPathsIterator"
        check_ffi_error(ret_code, error_msg)

        self.ptr = iter_ptr

    def __iter__(self):
        return self

    def __next__(self) -> StringPath:
        string_path_ptr = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.string_paths_iterator_next(
            self.ptr, ctypes.byref(string_path_ptr)
        )
        error_msg = "`next` failed"
        check_ffi_error(ret_code, error_msg)

        if not string_path_ptr:
            raise StopIteration

        return StringPath(string_path_ptr)

    def done(self) -> bool:
        done = ctypes.c_size_t()

        ret_code = lib.string_paths_iterator_done(self.ptr, ctypes.byref(done))
        error_msg = "`done` failed"
        check_ffi_error(ret_code, error_msg)

        return bool(done)

    def __del__(self):
        if hasattr(self, "ptr"):
            lib.string_paths_iterator_destroy(self.ptr)
