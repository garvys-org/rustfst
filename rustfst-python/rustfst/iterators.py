import ctypes
from rustfst.utils import lib, check_ffi_error
from rustfst.tr import Tr

class TrsIterator:
    """
    TrsIterator(fst, state)
      This class is used for iterating over the trs leaving some state of an FST.
    """

    def __init__(self, fst, state):
        self._fst = fst  # reference fst to prolong its lifetime (prevent early gc)
        state = ctypes.c_size_t(state)
        iter_ptr = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.trs_iterator_new(fst._fst, state, ctypes.byref(iter_ptr))
        err_msg = "`__init__` failed"
        check_ffi_error(ret_code, err_msg)

        self._ptr = iter_ptr

    def done(self):
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

    def __next__(self):
        """ x.next() -> the next value, or raise StopIteration """
        if self.done():
            raise StopIteration

        tr_ptr = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.trs_iterator_next(self._ptr, ctypes.byref(tr_ptr))
        err_msg = "`next` failed"
        check_ffi_error(ret_code, err_msg)

        return Tr(ptr=tr_ptr)

    def reset(self):
        """
        reset(self)
            Resets the iterator to the initial position.
        """
        ret_code = lib.trs_iterator_reset(self._ptr)
        err_msg = "`reset` failed"
        check_ffi_error(ret_code, err_msg)

    def __iter__(self):
        """ x.__iter__() <==> iter(x) """
        return self

    def __repr__(self):
        """ x.__repr__() <==> repr(x) """
        return "<TrsIterator at 0x{:x}>".format(id(self))

    def __del__(self):
        lib.trs_iterator_destroy(self._ptr)

class StateIterator:
    pass