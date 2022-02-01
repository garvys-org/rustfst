from contextlib import contextmanager
from ctypes import (
    cdll,
    c_char_p,
    string_at,
    byref,
    Structure,
    POINTER,
    c_int32,
    c_uint8,
)

from pathlib import Path
from typing import Union

dylib_dir = Path(__file__).parent
dylib_path = list(dylib_dir.glob("*.so"))[0]
lib = cdll.LoadLibrary(str(dylib_path))

PathOrStr = Union[Path, str]


class CStringArray(Structure):
    _fields_ = [("data", POINTER(c_char_p)), ("size", c_int32)]

    def to_pylist(self):
        return [self.data[i].decode("utf8") for i in range(self.size)]


class CBuffer(Structure):
    _fields_ = [("data", POINTER(c_uint8)), ("size", c_int32)]


@contextmanager
def string_pointer(ptr):
    try:
        yield ptr
    finally:
        lib.rustfst_destroy_string(ptr)


def check_ffi_error(exit_code, error_context_msg):
    if exit_code != 0:
        with string_pointer(c_char_p()) as ptr:
            if lib.rustfst_ffi_get_last_error(byref(ptr)) == 0:
                ffi_error_message = string_at(ptr).decode("utf8")
            else:
                ffi_error_message = "see stderr"
        raise ValueError("%s: %s" % (error_context_msg, ffi_error_message))
