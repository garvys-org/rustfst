from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst import Fst
from rustfst.symbol_table import SymbolTable
from rustfst.drawing_config import DrawingConfig
from typing import Optional
from pathlib import Path


class ConstFst(Fst):
    def __init__(self, ptr=None):
        if ptr:
            self.ptr = ptr
        else:
            raise ValueError(
                "Const fst should be init with a pointer or loaded from a file"
            )
        super().__init__(self.ptr)

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

        ret_code = lib.const_fst_draw(
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
        Read a Fst at a given path.
        Args:
          filename: The string location of the input file.
        Returns:
          An FST.
        Raises:
          ValueError: Read failed.
        """
        fst = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.const_fst_from_path(
            ctypes.byref(fst), str(filename).encode("utf-8")
        )
        err_msg = f"Read failed. file: {filename}"
        check_ffi_error(ret_code, err_msg)

        return cls(ptr=fst)

    def write(self, filename: Path):
        """
        Serializes FST to a file.
        This method writes the FST to a file in binary format.
        Args:
          filename: The string location of the output file.
        Raises:
          ValueError: Write failed.
        """
        ret_code = lib.const_fst_write_file(self.ptr, str(filename).encode("utf-8"))
        err_msg = f"Write failed. file: {filename}"
        check_ffi_error(ret_code, err_msg)

    def equals(self, other: Fst) -> bool:
        """
        equals(self, other)
            Check if this Fst is equal to the other
        :param other: Fst instance
        :return: bool
        """
        is_equal = ctypes.c_size_t()

        ret_code = lib.const_fst_equals(self.ptr, other.ptr, ctypes.byref(is_equal))
        err_msg = "Error checking equality"
        check_ffi_error(ret_code, err_msg)

        return bool(is_equal.value)

    def copy(self) -> ConstFst:
        """
        copy fst(self, other)
        :return: Fst
        """
        cloned_fst = ctypes.c_size_t()
        ret_code = lib.const_fst_copy(self.ptr, ctypes.byref(cloned_fst))
        err_msg = "Error copying fst"
        check_ffi_error(ret_code, err_msg)

        return ConstFst(cloned_fst)

    def __str__(self):
        s = ctypes.c_void_p()
        ret_code = lib.const_fst_display(self.ptr, ctypes.byref(s))
        err_msg = "Error displaying ConstFst"
        check_ffi_error(ret_code, err_msg)

        return ctypes.string_at(s).decode("utf8")
