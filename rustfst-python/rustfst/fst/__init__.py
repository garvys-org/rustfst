from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.symbol_table import SymbolTable
from rustfst.iterators import TrsIterator
from typing import Optional


class Fst:
    """
    Fst(ptr=None)
      This class wraps a mutable FST and exposes all methods.
      Args:
        ptr: An optional pointer pointing to an existing Fst rust struct.
    """

    def __init__(self, ptr, isymt=None, osymt=None):
        self.ptr = ptr
        self._input_symbols = isymt
        self._output_symbols = osymt

    def start(self) -> Optional[int]:
        """
        start(self)
            Returns the start state.
        """
        start = ctypes.c_size_t()
        ret_code = lib.fst_start(self.ptr, ctypes.byref(start))
        err_msg = "Error getting start state"
        check_ffi_error(ret_code, err_msg)

        if start is None:
            return None
        return int(start.value)

    def final(self, state: int) -> Optional[float]:
        """
        final(self, state)
            Returns the final weight of a state.
            Args:
              state: The integer index of a state.
            Returns:
              The final Weight of that state.
            Raises:
              Exception: If State index out of range.
        """
        state = ctypes.c_size_t(state)
        weight = ctypes.c_float()

        ret_code = lib.fst_final_weight(self.ptr, state, ctypes.byref(weight))
        err_msg = "Error getting final weight"
        check_ffi_error(ret_code, err_msg)

        if weight is None:
            return None

        return weight.value

    def num_trs(self, state: int) -> int:
        """
        num_trs(self, state)
            Returns the number of trs leaving a state.
            Args:
              state: The integer index of a state.
            Returns:
              The number of trs leaving that state.
            Raises:
              Exception: If State index out of range.
            See also: `num_states`.
        """
        num_trs = ctypes.c_size_t()
        state = ctypes.c_size_t(state)
        ret_code = lib.fst_num_trs(self.ptr, state, ctypes.byref(num_trs))
        err_msg = "Error getting number of trs"
        check_ffi_error(ret_code, err_msg)

        return int(num_trs.value)

    def trs(self, state: int) -> TrsIterator:
        """
        Returns an iterator over trs leaving the specified state.
        Args:
          state: The source state ID.
        Returns:
          An TrsIterator.
        See also: `mutable_trs`, `states`.
        """
        return TrsIterator(self, state)

    def is_final(self, state_id: int) -> bool:
        """
        is_final(state)
        Check if a state is final
        :param state_id:
        :return: bool
        """
        state = ctypes.c_size_t(state_id)
        is_final = ctypes.c_size_t()

        ret_code = lib.fst_is_final(self.ptr, state, ctypes.byref(is_final))
        err_msg = "Error checking if state is final"
        check_ffi_error(ret_code, err_msg)

        return bool(is_final.value)

    def is_start(self, state_id: int) -> bool:
        """
        is_start(state)
        Check if a state is a start
        :param state_id:
        :return: bool
        """
        state = ctypes.c_size_t(state_id)
        is_start = ctypes.c_size_t()

        ret_code = lib.fst_is_start(self.ptr, state, ctypes.byref(is_start))
        err_msg = "Error checking if state is final"
        check_ffi_error(ret_code, err_msg)

        return bool(is_start.value)

    def input_symbols(self) -> Optional[SymbolTable]:
        """
        input_symbols(self)
            Returns the FST's input symbol table, or None if none is present.
            See also: `output_symbols`.
        """
        if self._input_symbols:
            return self._input_symbols

        table = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_input_symbols(self.ptr, ctypes.byref(table))
        err_msg = "Error getting input symbols"
        check_ffi_error(ret_code, err_msg)

        if table.contents:
            return SymbolTable(ptr=table)
        return None

    def output_symbols(self) -> Optional[SymbolTable]:
        """
        output_symbols(self)
            Returns the FST's output symbol table, or None if none is present.
            See also: `input_symbols`.
        """
        if self._output_symbols:
            return self._output_symbols

        table = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.fst_output_symbols(self.ptr, ctypes.byref(table))
        err_msg = "Error getting output symbols"
        check_ffi_error(ret_code, err_msg)

        if table.contents:
            return SymbolTable(ptr=table)
        return None

    def set_input_symbols(self, syms: Optional[SymbolTable]) -> Fst:
        """
        Sets the input symbol table.
        Passing None as a value will delete the input symbol table.
        Args:
          syms: A SymbolTable.
        Returns:
          self.
        See also: `set_output_symbols`.
        """
        if syms is None:
            ret_code = lib.fst_unset_input_symbols(self.ptr)
            err_msg = "Error unsetting input symbols"
            check_ffi_error(ret_code, err_msg)
            # detach symbol table from fst
            self._input_symbols = None
            return self

        table = syms.ptr
        ret_code = lib.fst_set_input_symbols(self.ptr, table)
        err_msg = "Error setting input symbols"
        check_ffi_error(ret_code, err_msg)

        # attach symbol table to fst (prevent early gc of syms)
        self._input_symbols = syms

        return self

    def set_output_symbols(self, syms: Optional[SymbolTable]) -> Fst:
        """
        Sets the output symbol table.
        Passing None as a value will delete the output symbol table.
        Args:
          syms: A SymbolTable.
        Returns:
          self.
        See also: `set_input_symbols`.
        """
        if syms is None:
            ret_code = lib.fst_unset_output_symbols(self.ptr)
            err_msg = "Error unsetting output symbols"
            check_ffi_error(ret_code, err_msg)
            # detach symbol table from fst
            self._output_symbols = None
            return self

        table = syms.ptr

        ret_code = lib.fst_set_output_symbols(self.ptr, table)
        err_msg = "Error setting output symbols"
        check_ffi_error(ret_code, err_msg)

        # attach symbol table to fst (prevent early gc of syms)
        self._output_symbols = syms

        return self

    def remove_input_symbols(self, symbols: list[int]) -> Fst:
        """
        remove_input_symbols(self, symbols)
            Args:
              symbols: List[int]
            Returns:
              self.
        """
        symbols_ptr = (ctypes.c_int * len(symbols))(*symbols)
        symbols_len = ctypes.c_size_t(len(symbols))
        ret_code = lib.fst_remove_input_symbols(self.ptr, symbols_ptr, symbols_len)
        err_msg = "Error during remove_input_symbols"
        check_ffi_error(ret_code, err_msg)

        return self

    def remove_output_symbols(self, symbols: list[int]) -> Fst:
        """
        remove_output_symbols(self, symbols)
            Args:
              symbols: List[int]
            Returns:
              self.
        """
        symbols_ptr = (ctypes.c_int * len(symbols))(*symbols)
        symbols_len = ctypes.c_size_t(len(symbols))
        ret_code = lib.fst_remove_output_symbols(self.ptr, symbols_ptr, symbols_len)
        err_msg = "Error during remove_outout_symbols"
        check_ffi_error(ret_code, err_msg)

        return self

    def __eq__(self, y: Fst):
        """x.__eq__(y) <==> x==y"""
        return self.equals(y)

    def __repr__(self):
        return f"<rustfst.fst.Fst at {id(self)}>"

    def __del__(self):
        lib.fst_destroy(self.ptr)
