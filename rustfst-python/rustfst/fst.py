from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.drawing_config import DrawingConfig
from rustfst.symbol_table import SymbolTable
from rustfst.iterators import TrsIterator, MutableTrsIterator, StateIterator
from rustfst.tr import Tr
from rustfst.weight import weight_one
from typing import Optional
from pathlib import Path


class Fst:
    """
    Fst(ptr=None)
      This class wraps a mutable FST and exposes all methods.
      Args:
        ptr: An optional pointer pointing to an existing Fst rust struct.
    """

    def __init__(self, ptr=None) -> Fst:
        if ptr:
            self._fst = ptr
        else:
            fst_ptr = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.fst_new(ctypes.byref(fst_ptr))

            err_msg = "Something went wrong when creating the Fst struct"
            check_ffi_error(ret_code, err_msg)
            self._fst = fst_ptr

        # add shims for symbol tables (prevent early gc of the tables)
        self._input_symbols = None
        self._output_symbols = None

    @property
    def ptr(self):
        return self._fst

    def add_tr(self, state: int, tr: Tr) -> Fst:
        """
        add_tr(self, state, tr)
            Adds a new tr to the FST and return self. Note the tr should be considered
            consumed and is not safe to use it after.
            Args:
              state: The integer index of the source state.
              tr: The tr to add.
            Returns:
              self.
            Raises:
              SnipsFstException: If State index out of range.
            See also: `add_state`.
        """
        ret_code = lib.fst_add_tr(self._fst, ctypes.c_size_t(state), tr.ptr)
        err_msg = "Error during `add_tr`"
        check_ffi_error(ret_code, err_msg)

        return self

    def add_state(self) -> int:
        """
        add_state(self)
            Adds a new state to the FST and returns the state ID.
            Returns:
              The integer index of the new state.
            See also: `add_tr`, `set_start`, `set_final`.
        """
        state_id = ctypes.c_size_t()

        ret_code = lib.fst_add_state(self._fst, ctypes.byref(state_id))
        err_msg = "Error during `add_state`"
        check_ffi_error(ret_code, err_msg)

        return state_id.value

    def trs(self, state: int) -> TrsIterator:
        """
        trs(self, state)
            Returns an iterator over trs leaving the specified state.
            Args:
              state: The source state ID.
            Returns:
              An TrsIterator.
            See also: `mutable_trs`, `states`.
        """
        return TrsIterator(self, state)

    def mutable_trs(self, state: int) -> MutableTrsIterator:
        """
        mutable_trs(self, state)
            Returns a mutable iterator over trs leaving the specified state.
            Args:
              state: The source state ID.
            Returns:
              A MutableTrsIterator.
            See also: `trs`, `states`.
        """
        return MutableTrsIterator(self, state)

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

        ret_code = lib.fst_final_weight(self._fst, state, ctypes.byref(weight))
        err_msg = "Error getting final weight"
        check_ffi_error(ret_code, err_msg)

        if weight is None:
            return None

        return weight.value

    def delete_states(self):
        """
        delete_states(self)
            Delete the states
        """
        ret_code = lib.fst_delete_states(self._fst)
        err_msg = "Error deleting states"
        check_ffi_error(ret_code, err_msg)

    def input_symbols(self) -> Optional[SymbolTable]:
        """
        input_symbols(self)
            Returns the FST's input symbol table, or None if none is present.
            See also: `input_symbols`.
        """
        if self._input_symbols:
            return self._input_symbols

        table = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.fst_input_symbols(self._fst, ctypes.byref(table))
        err_msg = "Error getting input symbols"
        check_ffi_error(ret_code, err_msg)

        if table.contents:
            return SymbolTable(ptr=table)
        return None

    def is_final(self, state_id: int) -> bool:
        """
        is_final(state)
        Check wether state is final
        :param state_id:
        :return: bool
        """
        state = ctypes.c_size_t(state_id)
        is_final = ctypes.c_size_t()

        ret_code = lib.fst_is_final(self._fst, state, ctypes.byref(is_final))
        err_msg = "Error checking if state is final"
        check_ffi_error(ret_code, err_msg)

        return bool(is_final.value)

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
        ret_code = lib.fst_num_trs(self._fst, state, ctypes.byref(num_trs))
        err_msg = "Error getting number of trs"
        check_ffi_error(ret_code, err_msg)

        return int(num_trs.value)

    def num_states(self) -> int:
        """
        num_states(self)
            Returns the number of states.
        """
        num_states = ctypes.c_size_t()
        ret_code = lib.fst_num_states(self._fst, ctypes.byref(num_states))
        err_msg = "Error getting number of states"
        check_ffi_error(ret_code, err_msg)

        return int(num_states.value)

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
        ret_code = lib.fst_remove_input_symbols(self._fst, symbols_ptr, symbols_len)
        err_msg = "Error during remove_input_symbols"
        check_ffi_error(ret_code, err_msg)

        return self

    def output_symbols(self) -> Optional[SymbolTable]:
        """
        output_symbols(self)
            Returns the FST's output symbol table, or None if none is present.
            See also: `input_symbols`.
        """
        if self._output_symbols:
            return self._output_symbols

        table = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.fst_output_symbols(self._fst, ctypes.byref(table))
        err_msg = "Error getting output symbols"
        check_ffi_error(ret_code, err_msg)

        if table.contents:
            return SymbolTable(ptr=table)
        return None

    def set_final(self, state: int, weight: float = None):
        """
        set_final(self, state, weight)
            Sets the final weight for a state.
            Args:
              state: The integer index of a state.
              weight: A float indicating the desired final weight; if
                  omitted, it is set to semiring One.
            Raises:
              errors.FstException: State index out of range or Incompatible or invalid weight.
            See also: `set_start`.
        """
        if weight is None:
            weight = weight_one()

        state = ctypes.c_size_t(state)
        weight = ctypes.c_float(weight)

        ret_code = lib.fst_set_final(self._fst, state, ctypes.byref(weight))
        err_msg = "Error setting final state"
        check_ffi_error(ret_code, err_msg)

    def set_input_symbols(self, syms: SymbolTable) -> Fst:
        """
        set_input_symbols(self, syms)
            Sets the input symbol table.
            Passing None as a value will delete the input symbol table.
            Args:
              syms: A SymbolTable.
            Returns:
              self.
            See also: `set_output_symbols`.
        """
        if syms is None:
            ret_code = lib.fst_unset_input_symbols(self._fst)
            err_msg = "Error unsetting input symbols"
            check_ffi_error(ret_code, err_msg)
            # detach symbol table from fst
            self._input_symbols = None
            return self

        table = syms.ptr
        ret_code = lib.fst_set_input_symbols(self._fst, table)
        err_msg = "Error setting input symbols"
        check_ffi_error(ret_code, err_msg)

        # attach symbol table to fst (prevent early gc of syms)
        self._input_symbols = syms

        return self

    def set_output_symbols(self, syms: SymbolTable) -> Fst:
        """
        set_output_symbols(self, syms)
            Sets the output symbol table.
            Passing None as a value will delete the output symbol table.
            Args:
              syms: A SymbolTable.
            Returns:
              self.
            See also: `set_input_symbols`.
        """
        if syms is None:
            ret_code = lib.snips_fst_unset_output_symbols(self._fst)
            err_msg = "Error unsetting output symbols"
            check_ffi_error(ret_code, err_msg)
            # detach symbol table from fst
            self._output_symbols = None
            return self

        table = syms.ptr

        ret_code = lib.fst_set_output_symbols(self._fst, table)
        err_msg = "Error setting output symbols"
        check_ffi_error(ret_code, err_msg)

        # attach symbol table to fst (prevent early gc of syms)
        self._output_symbols = syms

        return self

    def set_start(self, state: int):
        """
        set_start(self, state)
            Sets a state to be the initial state state.
            Args:
              state: The integer index of a state.
            Returns:
              self.
            Raises:
              SnipsFstException: If State index out of range.
            See also: `set_final`.
        """
        state_id = ctypes.c_size_t(state)
        ret_code = lib.fst_set_start(self._fst, state_id)
        err_msg = "Error setting start state"
        check_ffi_error(ret_code, err_msg)

    def start(self) -> Optional[int]:
        """
        start(self)
            Returns the start state.
        """
        start = ctypes.c_size_t()
        ret_code = lib.fst_start(self._fst, ctypes.byref(start))
        err_msg = "Error getting start state"
        check_ffi_error(ret_code, err_msg)

        if start is None:
            return None
        return int(start.value)

    def states(self) -> StateIterator:
        """
        states(self)
            Returns an iterator over all states in the FST.
            Returns:
              A StateIterator object for the FST.
            See also: `trs`, `mutable_trs`.
        """
        return StateIterator(self)

    def draw(
        self,
        filename: str,
        isymbols: Optional[SymbolTable] = None,
        osymbols: Optional[SymbolTable] = None,
        drawing_config: DrawingConfig = DrawingConfig(),
    ):
        """
        draw(self, filename, isymbols=None, osymbols=None, ssymbols=None,
             acceptor=False, title="", width=8.5, height=11, portrait=False,
             vertical=False, ranksep=0.4, nodesep=0.25, fontsize=14,
             precision=5, show_weight_one=False, print_weight=True):
        Writes out the FST in Graphviz text format.
        This method writes out the FST in the dot graph description language. The
        graph can be rendered using the `dot` executable provided by Graphviz.
        Args:
          filename: The string location of the output dot/Graphviz file.
          isymbols: An optional symbol table used to label input symbols.
          osymbols: An optional symbol table used to label output symbols.
          drawing_config: Drawing configuration to use.
        See also: `text`.
        """
        isymbols = isymbols or self.input_symbols()
        osymbols = osymbols or self.output_symbols()

        isymbols_ptr = isymbols.ptr if isymbols is not None else None
        osymbols_ptr = osymbols.ptr if osymbols is not None else None

        if drawing_config.width is None:
            width = ctypes.c_float(-1.0)
        else:
            width = ctypes.c_float(drawing_config.width)

        if drawing_config.height is None:
            height = ctypes.c_float(-1.0)
        else:
            height = ctypes.c_float(drawing_config.height)

        if drawing_config.ranksep is None:
            ranksep = ctypes.c_float(-1.0)
        else:
            ranksep = ctypes.c_float(drawing_config.ranksep)

        if drawing_config.nodesep is None:
            nodesep = ctypes.c_float(-1.0)
        else:
            nodesep = ctypes.c_float(drawing_config.nodesep)

        ret_code = lib.fst_draw(
            self._fst,
            isymbols_ptr,
            osymbols_ptr,
            filename.encode("utf-8"),
            drawing_config.title.encode("utf-8"),
            ctypes.c_size_t(drawing_config.acceptor),
            width,
            height,
            ctypes.c_size_t(drawing_config.portrait),
            ctypes.c_size_t(drawing_config.vertical),
            ranksep,
            nodesep,
            ctypes.c_size_t(drawing_config.fontsize),
            ctypes.c_size_t(drawing_config.show_weight_one),
            ctypes.c_size_t(drawing_config.print_weight),
        )

        err_msg = "fst draw failed"
        check_ffi_error(ret_code, err_msg)

    @classmethod
    def read(cls, filename: Path) -> Fst:
        """
        Fst.read(filename)
            Reads an FST from a file.
            Args:
              filename: The string location of the input file.
            Returns:
              An FST.
            Raises:
              errors.SnipsFstException: Read failed.
        """
        fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_from_path(ctypes.byref(fst), str(filename).encode("utf-8"))
        err_msg = "Read failed. file: {}".format(filename)
        check_ffi_error(ret_code, err_msg)

        return cls(ptr=fst)

    def write(self, filename: Path):
        """
        write(self, filename)
            Serializes FST to a file.
            This method writes the FST to a file in vector binary format.
            Args:
              filename: The string location of the output file.
            Raises:
              errors.SnipsFstException: Write failed.
        """
        ret_code = lib.fst_write_file(self._fst, str(filename).encode("utf-8"))
        err_msg = "Write failed. file: {}".format(filename)
        check_ffi_error(ret_code, err_msg)

    def equals(self, other: Fst) -> bool:
        """
        equals(self, other)
            Check if this Fst is equal to the other
        :param other: Fst instance
        :return: bool
        """
        is_equal = ctypes.c_size_t()

        ret_code = lib.fst_equals(self._fst, other.ptr, ctypes.byref(is_equal))
        err_msg = "Error checking equality"
        check_ffi_error(ret_code, err_msg)

        return bool(is_equal.value)

    def __eq__(self, y: Fst):
        """x.__eq__(y) <==> x==y"""
        return self.equals(y)

    def __str__(self):
        return self.text()

    def __repr__(self):
        return "<rustfst.fst.Fst at {}>".format(id(self))

    def __del__(self):
        lib.fst_destroy(self._fst)
