from rustfst.ffi_utils import lib, check_ffi_error
import ctypes


class StringPath:
    def __init__(self, ptr):
        self.ptr = ptr

    def weight(self) -> float:
        weight = ctypes.c_float()
        ret_code = lib.string_path_weight(self.ptr, ctypes.byref(weight))
        error_msg = "`weight` failed"
        check_ffi_error(ret_code, error_msg)

        return weight.value

    def istring(self) -> str:
        istring = ctypes.c_void_p()
        ret_code = lib.string_path_istring(self.ptr, ctypes.byref(istring))
        error_msg = "`string` failed"
        check_ffi_error(ret_code, error_msg)

        return ctypes.string_at(istring).decode("utf8")

    def ostring(self) -> str:
        ostring = ctypes.c_void_p()
        ret_code = lib.string_path_ostring(self.ptr, ctypes.byref(ostring))
        error_msg = "`string` failed"
        check_ffi_error(ret_code, error_msg)

        return ctypes.string_at(ostring).decode("utf8")

    def __del__(self):
        lib.string_path_destroy(self.ptr)
