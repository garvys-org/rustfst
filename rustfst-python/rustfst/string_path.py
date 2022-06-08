from rustfst.ffi_utils import lib, check_ffi_error
import ctypes


class StringPath:
    """
    Struct representing a Path recognized by an Fst allowing to retrieve the input string,
    the output string and the weight of the Path.
    """

    def __init__(self, ptr):
        self.ptr = ptr

    def weight(self) -> float:
        """
        Returns the weight of the path.

        Returns:
            Weight of the path.
        """
        weight = ctypes.c_float()
        ret_code = lib.string_path_weight(self.ptr, ctypes.byref(weight))
        error_msg = "`weight` failed"
        check_ffi_error(ret_code, error_msg)

        return weight.value

    def istring(self) -> str:
        """
        Returns the input string of the Path.

        Returns:
            Input string of the Path.
        """
        istring = ctypes.c_void_p()
        ret_code = lib.string_path_istring(self.ptr, ctypes.byref(istring))
        error_msg = "`istring` failed"
        check_ffi_error(ret_code, error_msg)

        return ctypes.string_at(istring).decode("utf8")

    def ostring(self) -> str:
        """
        Returns the output string of the Path.

        Returns:
            Output string of the Path.
        """
        ostring = ctypes.c_void_p()
        ret_code = lib.string_path_ostring(self.ptr, ctypes.byref(ostring))
        error_msg = "`ostring` failed"
        check_ffi_error(ret_code, error_msg)

        return ctypes.string_at(ostring).decode("utf8")

    def __del__(self):
        lib.string_path_destroy(self.ptr)
