from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst import Fst
from rustfst.symbol_table import SymbolTable
from rustfst.drawing_config import DrawingConfig
from rustfst.iterators import MutableTrsIterator, StateIterator
from rustfst.tr import Tr
from rustfst.weight import weight_one
from typing import Optional
from pathlib import Path

from typing import List


class VectorFst(Fst):
    def __init__(self, ptr=None):
        if ptr:
            self.ptr = ptr
        else:
            fst_ptr = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.vec_fst_new(ctypes.byref(fst_ptr))

            err_msg = "Something went wrong when creating the Fst struct"
            check_ffi_error(ret_code, err_msg)
            self.ptr = fst_ptr
        super().__init__(self.ptr)

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
        ret_code = lib.vec_fst_add_tr(self.ptr, ctypes.c_size_t(state), tr.ptr)
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

        ret_code = lib.vec_fst_add_state(self.ptr, ctypes.byref(state_id))
        err_msg = "Error during `add_state`"
        check_ffi_error(ret_code, err_msg)

        return state_id.value

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

        ret_code = lib.vec_fst_set_final(self.ptr, state, weight)
        err_msg = "Error setting final state"
        check_ffi_error(ret_code, err_msg)

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

    def delete_states(self):
        """
        delete_states(self)
            Delete the states
        """
        ret_code = lib.vec_fst_delete_states(self.ptr)
        err_msg = "Error deleting states"
        check_ffi_error(ret_code, err_msg)

    def num_states(self) -> int:
        """
        num_states(self)
            Returns the number of states.
        """
        num_states = ctypes.c_size_t()
        ret_code = lib.vec_fst_num_states(self.ptr, ctypes.byref(num_states))
        err_msg = "Error getting number of states"
        check_ffi_error(ret_code, err_msg)

        return int(num_states.value)

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
        ret_code = lib.vec_fst_set_start(self.ptr, state_id)
        err_msg = "Error setting start state"
        check_ffi_error(ret_code, err_msg)

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

        ret_code = lib.vec_fst_draw(
            self.ptr,
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
            Read a Fst at a given path.
            Args:
              filename: The string location of the input file.
            Returns:
              An FST.
            Raises:
              errors.SnipsFstException: Read failed.
        """
        fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.vec_fst_from_path(
            ctypes.byref(fst), str(filename).encode("utf-8")
        )
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
        ret_code = lib.vec_fst_write_file(self.ptr, str(filename).encode("utf-8"))
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

        ret_code = lib.vec_fst_equals(self.ptr, other.ptr, ctypes.byref(is_equal))
        err_msg = "Error checking equality"
        check_ffi_error(ret_code, err_msg)

        return bool(is_equal.value)

    def copy(self) -> VectorFst:
        """
        copy fst(self, other)
        :return: Fst
        """
        cloned_fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.vec_fst_copy(self.ptr, ctypes.byref(cloned_fst))
        err_msg = "Error copying fst"
        check_ffi_error(ret_code, err_msg)

        return VectorFst(cloned_fst)

    def compose(self, other: VectorFst, config=None):
        from rustfst.algorithms.compose import compose, compose_with_config

        if config:
            return compose_with_config(self, other, config)
        return compose(self, other)

    def concat(self, other: VectorFst):
        from rustfst.algorithms.concat import concat

        return concat(self, other)

    def connect(self):
        from rustfst.algorithms.connect import connect

        connect(self)

    def determinize(self, config=None):
        from rustfst.algorithms.determinize import determinize, determinize_with_config

        if config:
            return determinize_with_config(self, config)
        return determinize(self)

    def project(self, proj_type=None):
        from rustfst.algorithms.project import project, ProjectType

        if proj_type:
            project(self, proj_type)
        proj_type = ProjectType.PROJECT_INPUT
        return project(self, proj_type)

    def replace(
        self,
        root_label: int,
        fst_list: List[(int, VectorFst)],
        epsilon_on_replace: bool = False,
    ):
        from rustfst.algorithms.replace import replace

        complete_fst_list = [(root_label, self)] + fst_list
        return replace(root_label, complete_fst_list, epsilon_on_replace)

    def reverse(self):
        from rustfst.algorithms.reverse import reverse

        return reverse(self)

    def rm_epsilon(self):
        from rustfst.algorithms.rm_epsilon import rm_epsilon

        rm_epsilon(self)

    def shortest_path(self, config=None):
        from rustfst.algorithms.shortest_path import (
            shortestpath,
            shortestpath_with_config,
        )

        if config:
            return shortestpath_with_config(self, config)
        return shortestpath(self)

    def union(self, other_fst: VectorFst):
        from rustfst.algorithms.union import union

        return union(self, other_fst)

    def optimize(self):
        from rustfst.algorithms.optimize import optimize

        optimize(self)

    def tr_sort(self, ilabel_cmp: bool = True):
        from rustfst.algorithms.tr_sort import tr_sort

        tr_sort(self, ilabel_cmp)

    def tr_unique(self):
        from rustfst.algorithms.tr_unique import tr_unique

        tr_unique(self)

    def isomorphic(self, other: VectorFst):
        from rustfst.algorithms.isomorphic import isomorphic

        return isomorphic(self, other)

    def __add__(self, y):
        """x.__add__(y) <==> x+y"""
        x = self.copy()

        return x.concat(y)
