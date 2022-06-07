from __future__ import annotations

from typing import Optional

from rustfst.weight import weight_one
from rustfst.fst.vector_fst import VectorFst
from rustfst.symbol_table import SymbolTable

import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)


def acceptor(
    astring: str, symbol_table: SymbolTable, weight: Optional[float] = None
) -> VectorFst:
    """
    Creates an acceptor from a string.
    This function creates a FST which accepts its input with a fixed weight
    (defaulting to semiring One).
    Args:
      astring: The input string.
      weight: A Weight or weight string indicating the desired path weight. If
        omitted or null, the path weight is set to semiring One.
      symbol_table: SymbolTable to be used to encode the string.
    Returns:
      An FST acceptor.
    """
    if weight is None:
        weight = weight_one()

    acceptor_fst_ptr = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.utils_string_to_acceptor(
        astring.encode("utf-8"),
        symbol_table.ptr,
        ctypes.c_float(weight),
        ctypes.byref(acceptor_fst_ptr),
    )
    err_msg = "Error creating acceptor FST"
    check_ffi_error(ret_code, err_msg)
    return VectorFst(ptr=acceptor_fst_ptr)


def transducer(
    istring: str,
    ostring: str,
    isymt: SymbolTable,
    osymt: SymbolTable,
    weight: Optional[float] = None,
) -> VectorFst:
    """
    Creates a transducer from a pair of strings or acceptor FSTs.
    This function creates a FST which transduces from the first string to
    the second with a fixed weight (defaulting to semiring One).
    Args:
      istring: The input string
      ostring: The output string
      weight: A Weight as float.
      isymt: SymbolTable to be used to encode the string.
      osymt: SymbolTable to be used to encode the string.
    Returns:
      An FST transducer.
    """

    if weight is None:
        weight = weight_one()

    transducer_fst_ptr = ctypes.c_void_p()
    ret_code = lib.utils_string_to_transducer(
        istring.encode("utf-8"),
        ostring.encode("utf-8"),
        isymt.ptr,
        osymt.ptr,
        ctypes.c_float(weight),
        ctypes.byref(transducer_fst_ptr),
    )
    err_msg = "Error creating tranducer FST"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=transducer_fst_ptr)


def epsilon_machine(weight: Optional[float] = None) -> VectorFst:
    """
    Constructs a single-state, no-arc FST accepting epsilon.
    This function creates an unweighted FST with a single state which is both
    initial and final.
    Args:
      weight: A Weight. Default semiring One.
    Returns:
      An FST.
    """
    if weight is None:
        weight = weight_one()
    fst = VectorFst()
    state = fst.add_state()
    fst.set_start(state)
    fst.set_final(state, weight)

    return fst
