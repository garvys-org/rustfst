from __future__ import annotations
from ctypes import (
    c_size_t,
    byref,
    c_float,
    c_void_p,
)
from rustfst.weight import weight_one
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from typing import Optional


class Tr:
    """
    Structure representing a transition from a state to another state in a FST.
    Attributes:
        ilabel: The input label.
        olabel: The output label.
        weight: The arc weight.
        nextstate: The destination state for the arc.
    """

    def __init__(
        self,
        ilabel: Optional[int] = None,
        olabel: Optional[int] = None,
        weight: Optional[float] = None,
        nextstate: Optional[int] = None,
    ):
        """
        Create a new transition.

        Args:
            ilabel: The input label.
            olabel: The outpit label.
            weight: The transition's weight
            nextstate: The destination state for the transition.
        """
        if ilabel and olabel is None and weight is None and nextstate is None:
            self._ptr = ilabel
        else:
            if weight is None:
                weight = weight_one()

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
    def ptr(self):
        return self._ptr

    @property
    def ilabel(self) -> int:
        ilabel = c_size_t()
        exit_code = lib.tr_ilabel(self._ptr, byref(ilabel))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return int(ilabel.value)

    @ilabel.setter
    def ilabel(self, value: int):
        ilabel = c_size_t(value)
        exit_code = lib.tr_set_ilabel(self._ptr, ilabel)
        err_msg = "Something went wrong when setting Tr ilabel value"
        check_ffi_error(exit_code, err_msg)

    @property
    def olabel(self) -> int:
        olabel = c_size_t()
        exit_code = lib.tr_olabel(self._ptr, byref(olabel))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return int(olabel.value)

    @olabel.setter
    def olabel(self, value: int):
        olabel = c_size_t(value)
        exit_code = lib.tr_set_olabel(self._ptr, olabel)
        err_msg = "Something went wrong when setting Tr olabel value"
        check_ffi_error(exit_code, err_msg)

    @property
    def weight(self) -> float:
        weight = c_float()
        exit_code = lib.tr_weight(self._ptr, byref(weight))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return weight.value

    @weight.setter
    def weight(self, value: float):
        weight = c_float(value)
        exit_code = lib.tr_set_weight(self._ptr, weight)
        err_msg = "Something went wrong when setting Tr weight value"
        check_ffi_error(exit_code, err_msg)

    @property
    def next_state(self) -> int:
        next_state = c_size_t()
        exit_code = lib.tr_next_state(self._ptr, byref(next_state))
        err_msg = "Something went wrong when reading Tr ilabel value"
        check_ffi_error(exit_code, err_msg)
        return int(next_state.value)

    @next_state.setter
    def next_state(self, next_state: int):
        next_state = c_size_t(next_state)
        exit_code = lib.tr_set_next_state(self._ptr, next_state)
        err_msg = "Something went wrong when setting Tr next_state value"
        check_ffi_error(exit_code, err_msg)

    def __eq__(self, other: Tr):
        return (
            self.ilabel == other.ilabel
            and self.olabel == other.olabel
            and self.weight == other.weight
            and self.next_state == other.next_state
        )

    def __repr__(self):
        """x.__repr__() <==> repr(x)"""
        return f"<Tr ilabel={self.ilabel}, olabel={self.olabel}, weight={self.weight}, next_state={self.next_state}>"

    def __del__(self):
        lib.tr_delete(self._ptr)
