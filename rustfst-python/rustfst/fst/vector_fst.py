from __future__ import annotations
import ctypes

from rustfst.string_paths_iterator import StringPathsIterator
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst import Fst
from rustfst.symbol_table import SymbolTable
from rustfst.drawing_config import DrawingConfig
from rustfst.iterators import MutableTrsIterator, StateIterator
from rustfst.tr import Tr
from rustfst.weight import weight_one
from typing import Optional, Union
from pathlib import Path

from typing import List


class VectorFst(Fst):
    def __init__(self, ptr=None):
        """
        Creates an empty VectorFst.
        """
        self._input_symbols = None
        self._output_symbols = None

        if ptr:
            self.ptr = ptr

            # Check if isymt inside
            isymt = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.fst_input_symbols(self.ptr, ctypes.byref(isymt))
            err_msg = "Error getting input symbols"
            check_ffi_error(ret_code, err_msg)
            if isymt.contents:
                self._input_symbols = SymbolTable(ptr=isymt)

            # Check if osymt inside
            osymt = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.fst_output_symbols(self.ptr, ctypes.byref(osymt))
            err_msg = "Error getting input symbols"
            check_ffi_error(ret_code, err_msg)
            if osymt.contents:
                self._output_symbols = SymbolTable(ptr=osymt)

        else:
            fst_ptr = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.vec_fst_new(ctypes.byref(fst_ptr))

            err_msg = "Something went wrong when creating the Fst struct"
            check_ffi_error(ret_code, err_msg)
            self.ptr = fst_ptr

        super().__init__(self.ptr, self._input_symbols, self._output_symbols)

    def add_tr(self, state: int, tr: Tr) -> Fst:
        """
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
        Sets the final weight for a state.
        Args:
          state: The integer index of a state.
          weight: A float indicating the desired final weight; if
              omitted, it is set to semiring One.
        Raises:
          ValueError: State index out of range or Incompatible or invalid weight.
        See also: `set_start`.
        """
        if weight is None:
            weight = weight_one()

        state = ctypes.c_size_t(state)
        weight = ctypes.c_float(weight)

        ret_code = lib.vec_fst_set_final(self.ptr, state, weight)
        err_msg = "Error setting final state"
        check_ffi_error(ret_code, err_msg)

    def unset_final(self, state: int):
        """
        Unset the final weight of a state. As a result, the state is no longer final.
        Args:
            state: The integer index of a state
        Raises:
          ValueError: State index out of range.
        """
        state = ctypes.c_size_t(state)
        ret_code = lib.vec_fst_del_final_weight(self.ptr, state)
        err_msg = "Error unsetting final state"
        check_ffi_error(ret_code, err_msg)

    def mutable_trs(self, state: int) -> MutableTrsIterator:
        """
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
        Delete all the states
        """
        ret_code = lib.vec_fst_delete_states(self.ptr)
        err_msg = "Error deleting states"
        check_ffi_error(ret_code, err_msg)

    def num_states(self) -> int:
        """
        Returns the number of states.
        Returns:
            Number of states present in the Fst.
        """
        num_states = ctypes.c_size_t()
        ret_code = lib.vec_fst_num_states(self.ptr, ctypes.byref(num_states))
        err_msg = "Error getting number of states"
        check_ffi_error(ret_code, err_msg)

        return int(num_states.value)

    def set_start(self, state: int):
        """
        Sets a state to be the initial state state.
        Args:
          state: The integer index of a state.
        Raises:
          ValueError: If State index out of range.
        See also: `set_final`.
        """
        state_id = ctypes.c_size_t(state)
        ret_code = lib.vec_fst_set_start(self.ptr, state_id)
        err_msg = "Error setting start state"
        check_ffi_error(ret_code, err_msg)

    def states(self) -> StateIterator:
        """
        Returns an iterator over all states in the FST.
        Returns:
          A StateIterator object for the FST.
        See also: `trs`, `mutable_trs`.
        """
        return StateIterator(self)

    def relabel_tables(
        self,
        *,
        old_isymbols: Optional[SymbolTable] = None,
        new_isymbols: SymbolTable,
        attach_new_isymbols: bool = True,
        old_osymbols: Optional[SymbolTable] = None,
        new_osymbols: SymbolTable,
        attach_new_osymbols: bool = True,
    ) -> VectorFst:
        """
        Destructively relabel the Fst with new Symbol Tables.

        Relabelling refers to the operation where all the labels of an Fst are mapped to the equivalent labels
        of a new `SymbolTable`.
        If the Fst has a label `1` corresponding to the symbol "alpha" in the current symbol table and "alpha"
        is mapped to 4 in a new SymbolTable, then all the 1 are going to be mapped to 4.

        Args:
            old_isymbols: Input `SymbolTable` used to build the Fst. If `None`, uses the Input `SymbolTable` attached to the Fst.
            new_isymbols: New Input `SymbolTable` to use.
            attach_new_isymbols: Whether to attach the new Input `SymbolTable` to the Fst. If False, the resulting Fst won't contain any attached Input `SymbolTable`.
            old_osymbols: Output `SymbolTable` used to build the Fst. If `None`, uses the Output `SymbolTable` attached to the Fst
            new_osymbols: New Output `SymbolTable` to use.
            attach_new_osymbols: Whether to attach the new Output `SymbolTable` to the Fst. If False, the resulting Fst won't contain any attached Output `SymbolTable`.

        Returns:
            self

        """
        old_isymbols_ptr = old_isymbols.ptr if old_isymbols is not None else None
        old_osymbols_ptr = old_osymbols.ptr if old_osymbols is not None else None

        ret_code = lib.vec_fst_relabel_tables(
            self.ptr,
            old_isymbols_ptr,
            new_isymbols.ptr,
            ctypes.c_size_t(attach_new_isymbols),
            old_osymbols_ptr,
            new_osymbols.ptr,
            ctypes.c_size_t(attach_new_osymbols),
        )
        err_msg = "Relabel tables failed"
        check_ffi_error(ret_code, err_msg)

        # Necessary because the symts are cached on the python side.
        if attach_new_isymbols:
            self._input_symbols = new_isymbols
        else:
            self._input_symbols = None

        if attach_new_osymbols:
            self._output_symbols = new_osymbols
        else:
            self._output_symbols = None

        return self

    def draw(
        self,
        filename: str,
        isymbols: Optional[SymbolTable] = None,
        osymbols: Optional[SymbolTable] = None,
        drawing_config: DrawingConfig = DrawingConfig(),
    ):
        """
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
    def read(cls, filename: Union[str, Path]) -> VectorFst:
        """
        Read a Fst at a given path.
        Args:
          filename: The string location of the input file.
        Returns:
          An Fst.
        Raises:
          ValueError: Read failed.
        """
        fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.vec_fst_from_path(
            ctypes.byref(fst), str(filename).encode("utf-8")
        )
        err_msg = f"Read failed. file: {filename}"
        check_ffi_error(ret_code, err_msg)

        return cls(ptr=fst)

    def write(self, filename: Union[str, Path]):
        """
        Serializes FST to a file.
        This method writes the FST to a file in vector binary format.
        Args:
          filename: The string location of the output file.
        Raises:
          ValueError: Write failed.
        """
        ret_code = lib.vec_fst_write_file(self.ptr, str(filename).encode("utf-8"))
        err_msg = f"Write failed. file: {filename}"
        check_ffi_error(ret_code, err_msg)

    @classmethod
    def from_bytes(cls, data: bytes) -> VectorFst:
        """
        Load a `VectorFst` from a sequence of bytes.
        Args:
            data: Sequence of bytes.

        Returns:
            Loaded `VectorFst`.
        """
        fst_ptr = ctypes.pointer(ctypes.c_void_p())

        # Define a temporary struct to hold the bytes array
        class BytesArray(ctypes.Structure):
            _fields_ = [("data_ptr", ctypes.c_char_p), ("size", ctypes.c_size_t)]

        c_bytes = BytesArray(data, len(data))

        ret_code = lib.vec_fst_from_bytes(ctypes.byref(c_bytes), ctypes.byref(fst_ptr))
        error_msg = "`from_bytes` failed"
        check_ffi_error(ret_code, error_msg)

        return VectorFst(ptr=fst_ptr)

    def to_bytes(self) -> bytes:
        """
        Turns the `VectorFst` into bytes.
        Returns:
            Sequence of bytes.
        """
        # Define a temporary struct to hold the bytes array
        class BytesArray(ctypes.Structure):
            _fields_ = [("data_ptr", ctypes.c_void_p), ("size", ctypes.c_size_t)]

        bytes_ptr = ctypes.pointer(BytesArray())

        ret_code = lib.vec_fst_to_bytes(self.ptr, ctypes.byref(bytes_ptr))
        error_msg = "`to_bytes` failed"
        check_ffi_error(ret_code, error_msg)

        return bytes(
            [
                ctypes.c_ubyte.from_address(bytes_ptr.contents.data_ptr + i).value
                for i in range(bytes_ptr.contents.size)
            ]
        )

    def equals(self, other: Fst) -> bool:
        """
        Check if this Fst is equal to the other.
        Args:
            other: Fst instance
        Returns:
             Whether both Fst are equals.
        """
        is_equal = ctypes.c_size_t()

        ret_code = lib.vec_fst_equals(self.ptr, other.ptr, ctypes.byref(is_equal))
        err_msg = "Error checking equality"
        check_ffi_error(ret_code, err_msg)

        return bool(is_equal.value)

    def copy(self) -> VectorFst:
        """
        Returns:
            A copy of the Fst.
        """
        cloned_fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.vec_fst_copy(self.ptr, ctypes.byref(cloned_fst))
        err_msg = "Error copying fst"
        check_ffi_error(ret_code, err_msg)

        return VectorFst(cloned_fst)

    def compose(self, other: VectorFst, config=None) -> VectorFst:
        from rustfst.algorithms.compose import compose, compose_with_config

        if config:
            return compose_with_config(self, other, config)
        return compose(self, other)

    def concat(self, other: VectorFst) -> VectorFst:
        """
        Compute Fst Concatenation of this Fst with another Fst. Returning the resulting Fst.
        Args:
            other: Fst to concatenate with.

        Returns:
            The concatenated Fst.

        """
        from rustfst.algorithms.concat import concat

        return concat(self, other)

    def connect(self) -> VectorFst:
        """
        This operation trims an Fst, removing states and trs that are not on successful paths.

        Examples :

        - Input

        ![connect_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_in.svg?sanitize=true)

        - Output

        ![connect_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_out.svg?sanitize=true)

        Returns:
            self
        """
        from rustfst.algorithms.connect import connect

        return connect(self)

    def top_sort(self) -> VectorFst:
        """
        This operation topologically sorts its input. When sorted, all transitions are from
        lower to higher state IDs.

        Examples :

        - Input

        ![topsort_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_in.svg?sanitize=true)

        - Output

        ![topsort_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/topsort_out.svg?sanitize=true)

        Returns:
            Equivalent top sorted Fst. Modification also happens in-place.
        """
        from rustfst.algorithms.top_sort import top_sort

        return top_sort(self)

    def determinize(self, config=None) -> VectorFst:
        from rustfst.algorithms.determinize import determinize, determinize_with_config

        if config:
            return determinize_with_config(self, config)
        return determinize(self)

    def project(self, proj_type=None) -> VectorFst:
        from rustfst.algorithms.project import project, ProjectType

        if proj_type:
            return project(self, proj_type)
        proj_type = ProjectType.PROJECT_INPUT
        return project(self, proj_type)

    def replace(
        self,
        root_label: int,
        fst_list: List[(int, VectorFst)],
        epsilon_on_replace: bool = False,
    ) -> VectorFst:
        from rustfst.algorithms.replace import replace

        complete_fst_list = [(root_label, self)] + fst_list
        return replace(root_label, complete_fst_list, epsilon_on_replace)

    def reverse(self) -> VectorFst:
        from rustfst.algorithms.reverse import reverse

        return reverse(self)

    def rm_epsilon(self):
        from rustfst.algorithms.rm_epsilon import rm_epsilon

        rm_epsilon(self)

    def shortest_path(self, config=None) -> VectorFst:
        from rustfst.algorithms.shortest_path import (
            shortestpath,
            shortestpath_with_config,
        )

        if config:
            return shortestpath_with_config(self, config)
        return shortestpath(self)

    def union(self, other_fst: VectorFst) -> VectorFst:
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

    def isomorphic(self, other: VectorFst) -> bool:
        from rustfst.algorithms.isomorphic import isomorphic

        return isomorphic(self, other)

    def __add__(self, other: VectorFst) -> VectorFst:
        """
        `fst_1 + fst_2` is a shortcut to perform the concatenation of `fst_1` and `fst_2`.
        Args:
            other: VectorFst to concatenate after the current Fst.

        Returns:
            The concatenated Fst.
        """
        x = self.copy()

        return x.concat(other)

    def __mul__(self, other: VectorFst) -> VectorFst:
        """
        `fst_1 * fst_2` is a shortcut to perform the composition of `fst_1` and `fst_2`.
        Args:
            other: VectorFst to compose with.

        Returns:
            The composed Fst.

        """
        return self.compose(other)

    def __or__(self, other: VectorFst) -> VectorFst:
        """
        `fst_1 | fst_2` is a shortcut to perform the union of `fst_1` and `fst_2`.
        Args:
            other: VectorFst to perform the union with.

        Returns:
            The resulting Fst.
        """
        x = self.copy()

        return x.union(other)

    def __str__(self):
        s = ctypes.c_void_p()
        ret_code = lib.vec_fst_display(self.ptr, ctypes.byref(s))
        err_msg = "Error displaying VectorFst"
        check_ffi_error(ret_code, err_msg)

        return ctypes.string_at(s).decode("utf8")

    def string_paths(self) -> StringPathsIterator:
        return StringPathsIterator(self)
