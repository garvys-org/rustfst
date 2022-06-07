from __future__ import annotations

from typing import Union

from rustfst.ffi_utils import lib, check_ffi_error
import ctypes
from pathlib import Path


class SymbolTable:
    """
    `SymbolTable` class. This class wraps the `SymbolTable` struct.
    """

    def __init__(self, ptr=None):
        """
        Creates an empty `SymbolTable`.
        """
        if ptr:
            self.ptr = ptr
        else:
            symt_ptr = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.symt_new(ctypes.byref(symt_ptr))
            err_msg = "__init__ failed"
            check_ffi_error(ret_code, err_msg)

            self.ptr = symt_ptr

    def add_symbol(self, symbol: str) -> int:
        """
        Adds a symbol to the table and returns the index.

        Args:
          symbol: A symbol unicode string.
        Returns:
          The integer key of the new symbol.
        """
        try:
            symbol = symbol.encode("utf-8")
        except UnicodeDecodeError:
            symbol = ctypes.c_char_p(symbol)

        integer_key = ctypes.c_size_t()
        ret_code = lib.symt_add_symbol(self.ptr, symbol, ctypes.byref(integer_key))
        err_msg = "`add_symbol` failed"
        check_ffi_error(ret_code, err_msg)

        return int(integer_key.value)

    def add_table(self, syms: SymbolTable):
        """
        This method merges another symbol table into the current table. All key
        values will be offset by the current available key.
        Args:
          syms: A `SymbolTable` to be merged with the current table.
        """
        ret_code = lib.symt_add_table(self.ptr, syms.ptr)
        err_msg = "`add_table` failed"
        check_ffi_error(ret_code, err_msg)

    def copy(self) -> SymbolTable:
        """
        Returns:
            A mutable copy of the `SymbolTable`.
        """
        clone = ctypes.pointer(ctypes.c_void_p())

        ret_code = lib.symt_copy(self.ptr, ctypes.byref(clone))
        err_msg = "`copy` failed."
        check_ffi_error(ret_code, err_msg)

        return SymbolTable(ptr=clone)

    def find(self, key: Union[int, str]) -> Union[int, str]:
        """
        Given a symbol or index, finds the other one.
        This method returns the index associated with a symbol key, or the symbol
        associated with a index key.
        Args:
          key: Either a string or an index.
        Returns:
          If key is a string, the associated index; if key is an integer, the
              associated symbol.
        Raises:
          KeyError: Key not found.
        """
        if isinstance(key, int):
            return self._find_index(key)
        if isinstance(key, str):
            return self._find_symbol(key)
        raise f"key can only be a string or integer. Not {type(key)}"

    def _find_index(self, key: int) -> str:
        key = ctypes.c_size_t(key)
        symbol = ctypes.c_void_p()
        ret_code = lib.symt_find_index(self.ptr, key, ctypes.byref(symbol))
        err_msg = "`find` failed"
        check_ffi_error(ret_code, err_msg)

        return ctypes.string_at(symbol).decode("utf8")

    def _find_symbol(self, symbol: str) -> int:
        symbol = symbol.encode("utf-8")
        index = ctypes.c_size_t()
        ret_code = lib.symt_find_symbol(self.ptr, symbol, ctypes.byref(index))
        err_msg = "`find` failed"
        check_ffi_error(ret_code, err_msg)

        return int(index.value)

    def member(self, key: Union[int, str]) -> bool:
        """
        Given a symbol or index, returns whether it is found in the table.
        This method returns a boolean indicating whether the given symbol or index
        is present in the table. If one intends to perform subsequent lookup, it is
        better to simply call the find method, catching the KeyError.
        Args:
          key: Either a string or an index.
        Returns:
          Whether or not the key is present (as a string or a index) in the table.
        """
        is_present = ctypes.c_size_t()

        ret_code = None

        if isinstance(key, int):
            index = ctypes.c_size_t(key)
            ret_code = lib.symt_member_index(self.ptr, index, ctypes.byref(is_present))
        elif isinstance(key, str):
            symbol = key.encode("utf-8")
            ret_code = lib.symt_member_symbol(
                self.ptr, symbol, ctypes.byref(is_present)
            )
        else:
            raise f"key can only be a string or integer. Not {type(key)}"

        err_msg = "`member` failed"
        check_ffi_error(ret_code, err_msg)

        return bool(is_present.value)

    def num_symbols(self) -> int:
        """
        Returns:
            The number of symbols in the symbol table.
        """
        num_symbols = ctypes.c_size_t()
        ret_code = lib.symt_num_symbols(self.ptr, ctypes.byref(num_symbols))
        err_msg = "`num_symbols` failed"
        check_ffi_error(ret_code, err_msg)

        return int(num_symbols.value)

    @classmethod
    def read(cls, filename: Path) -> SymbolTable:
        """
        Reads symbol table from binary file.
        This class method creates a new SymbolTable from a symbol table binary file.
        Args:
          filename: The string location of the input binary file.
        Returns:
          A new SymbolTable instance.
        See also: `SymbolTable.read_fst`, `SymbolTable.read_text`.
        """
        symt = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.symt_from_path(
            ctypes.byref(symt), str(filename).encode("utf-8"), ctypes.c_size_t(1)
        )

        err_msg = f"Read failed for bin file : {filename}"
        check_ffi_error(ret_code, err_msg)

        return cls(ptr=symt)

    @classmethod
    def read_text(cls, filename: Path) -> SymbolTable:
        """
        Reads symbol table from text file.
        This class method creates a new SymbolTable from a symbol table text file.
        Args:
          filename: The string location of the input text file.

        Returns:
          A new SymbolTable instance.
        See also: `SymbolTable.read`, `SymbolTable.read_fst`.
        """
        symt = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.symt_from_path(
            ctypes.byref(symt), str(filename).encode("utf-8"), ctypes.c_size_t(0)
        )

        err_msg = f"Read failed for text file : {filename}"
        check_ffi_error(ret_code, err_msg)

        return cls(ptr=symt)

    def write(self, filename: Path):
        """
        Serializes symbol table to a file.
        This methods writes the SymbolTable to a file in binary format.
        Args:
          filename: The string location of the output file.
        Raises:
          FstIOError: Write failed.
        """
        ret_code = lib.symt_write_file(
            self.ptr, str(filename).encode("utf-8"), ctypes.c_size_t(1)
        )

        err_msg = f"Write failed for bin file : {filename}"
        check_ffi_error(ret_code, err_msg)

    def write_text(self, filename: Path):
        """
        Writes symbol table to text file.
        This method writes the SymbolTable to a file in human-readable format.
        Args:
          filename: The string location of the output file.
        Raises:
          FstIOError: Write failed.
        """
        ret_code = lib.symt_write_file(
            self.ptr, str(filename).encode("utf-8"), ctypes.c_size_t(0)
        )

        err_msg = f"Write failed for text file : {filename}"
        check_ffi_error(ret_code, err_msg)

    def equals(self, other: SymbolTable) -> bool:
        """
        Check if this SymbolTable is equal to the other

        Params:
            other: SymbolTable instance
        Returns:
             bool
        """
        is_equal = ctypes.c_size_t()

        ret_code = lib.symt_equals(self.ptr, other.ptr, ctypes.byref(is_equal))
        err_msg = "Error checking equality"
        check_ffi_error(ret_code, err_msg)

        return bool(is_equal.value)

    def __eq__(self, other: SymbolTable) -> bool:
        """
        Check if this `SymbolTable` is equal to the other

        Params:
            other: SymbolTable instance
        Returns:
             bool
        """
        return self.equals(other)

    def __del__(self):
        lib.symt_destroy(self.ptr)
