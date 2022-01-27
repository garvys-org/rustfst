from __future__ import annotations
import ctypes
from typing import Optional
from rustfst.utils import lib, check_ffi_error
from rustfst.tr import Tr


class TrsIterator:
    """
    TrsIterator(fst, state)
      This class is used for iterating over the trs leaving some state of an FST.
    """

    def __init__(self, fst: Fst, state: int) -> TrsIterator:
        self._fst = fst  # reference fst to prolong its lifetime (prevent early gc)
        state = ctypes.c_size_t(state)
        iter_ptr = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.trs_iterator_new(fst._fst, state, ctypes.byref(iter_ptr))
        err_msg = "`__init__` failed"
        check_ffi_error(ret_code, err_msg)

        self._ptr = iter_ptr

    def done(self) -> bool:
        """
        done(self)
            Indicates whether the iterator is exhausted or not.
            Returns:
              True if the iterator is exhausted, False otherwise.
        """
        done = ctypes.c_size_t()

        ret_code = lib.trs_iterator_done(self._ptr, ctypes.byref(done))
        err_msg = "`done` failed"
        check_ffi_error(ret_code, err_msg)

        return bool(done.value)

    def __next__(self) -> Optional[Tr]:
        """x.next() -> the next value, or raise StopIteration"""
        if self.done():
            raise StopIteration

        tr_ptr = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.trs_iterator_next(self._ptr, ctypes.byref(tr_ptr))
        err_msg = "`next` failed"
        check_ffi_error(ret_code, err_msg)

        if tr_ptr is None:
            return None

        return Tr(ptr=tr_ptr)

    def reset(self):
        """
        reset(self)
            Resets the iterator to the initial position.
        """
        ret_code = lib.trs_iterator_reset(self._ptr)
        err_msg = "`reset` failed"
        check_ffi_error(ret_code, err_msg)

    def __iter__(self) -> TrsIterator:
        """x.__iter__() <==> iter(x)"""
        return self

    def __repr__(self):
        """x.__repr__() <==> repr(x)"""
        return "<TrsIterator at 0x{:x}>".format(id(self))

    def __del__(self):
        lib.trs_iterator_destroy(self._ptr)


class MutableTrsIterator:
    """
    MutableTrsIterator(ifst, state)
      This class is used for iterating over the trs leaving some state of an FST,
      also permitting mutation of the current tr.
    """

    def __init__(self, fst: Fst, state_id: int) -> MutableTrsIterator:
        self._fst = fst  # reference fst to prolong its lifetime (prevent early gc)
        state_id = ctypes.c_size_t(state_id)
        iter_ptr = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.mut_trs_iterator_new(fst._fst, state_id, ctypes.byref(iter_ptr))
        err_msg = "`__init__` failed"
        check_ffi_error(ret_code, err_msg)

        self._ptr = iter_ptr

    def done(self) -> bool:
        """
        done(self)
            Indicates whether the iterator is exhausted or not.
            Returns:
              True if the iterator is exhausted, False otherwise.
        """
        done = ctypes.c_size_t()

        ret_code = lib.mut_trs_iterator_done(self._ptr, ctypes.byref(done))
        err_msg = "`done` failed"
        check_ffi_error(ret_code, err_msg)

        return bool(done.value)

    def __next__(self):
        """
        Advances the internal tr iteractor.
        :return: None
        """
        ret_code = lib.mut_trs_iterator_next(self._ptr)
        err_msg = "`next` failed"
        check_ffi_error(ret_code, err_msg)

    def reset(self):
        """
        reset(self)
            Resets the iterator to the initial position.
        """
        ret_code = lib.mut_trs_iterator_reset(self._ptr)
        err_msg = "`reset`failed"
        check_ffi_error(ret_code, err_msg)

    def set_value(self, tr: Tr):
        """
        set_value(self, tr)
            Replace the current tr with a new tr.
            Args:
              tr: The tr to replace the current tr with.
        """
        ret_code = lib.mut_trs_iterator_set_value(self._ptr, tr.ptr)
        err_msg = "`set_value` failed"
        check_ffi_error(ret_code, err_msg)

    def value(self) -> Optional[Tr]:
        """
        value(self)
            Returns the current tr.
        """
        tr_ptr = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.mut_trs_iterator_value(self._ptr, ctypes.byref(tr_ptr))
        err_msg = "`value` failed"
        check_ffi_error(ret_code, err_msg)

        if tr_ptr is None:
            return None

        return Tr(ptr=tr_ptr)

    def __iter__(self) -> MutableTrsIterator:
        """x.__iter__() <==> iter(x)"""
        return self

    def __repr__(self):
        """x.__repr__() <==> repr(x)"""
        return "<MutableTrsIterator at 0x{:x}>".format(id(self))

    def __del__(self):
        lib.mut_trs_iterator_destroy(self._ptr)


class StateIterator:
    """
    StateIterator(fst)
      This class is used for iterating over the states in an FST.
    """

    def __init__(self, fst: Fst) -> StateIterator:
        self._fst = fst  # reference fst to prolong its lifetime (prevent early gc)
        iter_ptr = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.state_iterator_new(fst._fst, ctypes.byref(iter_ptr))
        err_msg = "`__init__` failed"
        check_ffi_error(ret_code, err_msg)

        self._ptr = iter_ptr

    def done(self) -> bool:
        """
        done(self)
            Indicates whether the iterator is exhausted or not.
            Returns:
              True if the iterator is exhausted, False otherwise.
        """
        done = ctypes.c_size_t()

        ret_code = lib.state_iterator_done(self._ptr, ctypes.byref(done))
        err_msg = "`done` failed"
        check_ffi_error(ret_code, err_msg)

        return bool(done.value)

    def __next__(self) -> Optional[int]:
        """x.next() -> the next value, or raise StopIteration"""
        if self.done():
            raise StopIteration

        next_state = ctypes.c_size_t()
        ret_code = lib.state_iterator_next(self._ptr, ctypes.byref(next_state))
        err_msg = "`next` failed"
        check_ffi_error(ret_code, err_msg)

        if next_state is None:
            return None
        return int(next_state.value)

    def __iter__(self) -> StateIterator:
        """x.__iter__() <==> iter(x)"""
        return self

    def __repr__(self):
        """x.__repr__() <==> repr(x)"""
        return "<StateIterator at 0x{:x}>".format(id(self))

    def __del__(self):
        lib.state_iterator_destroy(self._ptr)
