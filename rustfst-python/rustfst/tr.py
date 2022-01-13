from ctypes import (
    c_size_t,
    byref,
    c_float,
    c_void_p,
)
from rustfst.utils import (
    lib,
    check_ffi_error,
)


class Tr:
    """
    Tr(ilabel, olabel, weight, next_state)
        Structure representing a transition from a state to another state in a FST.
        Attributes:
            ilabel: The input label.
            olabel: The output label.
            weight: The arc weight.
            nextstate: The destination state for the arc.
    """

    def __init__(self, ptr=None, olabel=None, weight=None, nextstate=None):
        if ptr and olabel == None and weight == None and nextstate == None:
            self._ptr = ptr
        else:
            ilabel = ptr
            ptr = c_void_p()
            exit_code = lib.tr_new(
                c_size_t(ilabel),
                c_size_t(olabel),
                c_float(weight),
                c_size_t(nextstate),
                byref(ptr),
            )
            err_msg = "Something went wrong when creating the Tr struct"
            check_ffi_error(exit_code, err_msg)

            self._ptr = ptr

    @property
    def ilabel(self):
        ilabel = c_size_t()
        exit_code = lib.tr_ilabel(self._ptr, byref(ilabel))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return ilabel.value

    @ilabel.setter
    def ilabel(self, value):
        ilabel = c_size_t(value)
        exit_code = lib.tr_set_ilabel(self._ptr, ilabel)
        err_msg = "Something went wrong when setting Tr ilabel value"
        check_ffi_error(exit_code, err_msg)

    @property
    def olabel(self):
        olabel = c_size_t()
        exit_code = lib.tr_olabel(self._ptr, byref(olabel))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return olabel.value

    @olabel.setter
    def olabel(self, value):
        olabel = c_size_t(value)
        exit_code = lib.tr_set_olabel(self._ptr, olabel)
        err_msg = "Something went wrong when setting Tr olabel value"
        check_ffi_error(exit_code, err_msg)

    @property
    def weight(self):
        weight = c_float()
        exit_code = lib.tr_weight(self._ptr, byref(weight))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return weight.value

    @weight.setter
    def weight(self, value):
        weight = c_float(value)
        exit_code = lib.tr_set_weight(self._ptr, weight)
        err_msg = "Something went wrong when setting Tr weight value"
        check_ffi_error(exit_code, err_msg)

    @property
    def next_state(self):
        next_state = c_size_t()
        exit_code = lib.tr_next_state(self._ptr, byref(next_state))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return next_state.value

    @next_state.setter
    def next_state(self, next_state):
        next_state = c_size_t(next_state)
        exit_code = lib.tr_set_next_state(self._ptr, next_state)
        err_msg = "Something went wrong when setting Tr next_state value"
        check_ffi_error(exit_code, err_msg)

    def __eq__(self, other):
        return self.ilabel == other.ilabel \
        and self.olabel == other.olabel \
        and self.weight == other.weight \
        and self.next_state == other.next_state

    def __repr__(self):
        """x.__repr__() <==> repr(x)"""
        return "<Tr ilabel={}, olabel={}, weight={}, next_state={}>".format(
            self.ilabel, self.olabel, self.weight, self.next_state
        )

    def __del__(self):
        lib.tr_delete(self._ptr)