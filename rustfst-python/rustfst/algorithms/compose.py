from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


from enum import Enum


class ComposeFilter(Enum):
    AUTOFILTER = 0
    NULLFILTER = 1
    TRIVIALFILTER = 2
    SEQUENCEFILTER = 3
    ALTSEQUENCEFILTER = 4
    MATCHFILTER = 5
    NOMATCHFILTER = 6


class ComposeConfig:
    def __init__(self, compose_filter=None, connect: bool = None):
        if compose_filter and connect is None:
            self.ptr = compose_filter
        elif compose_filter and connect:
            config = ctypes.pointer(ctypes.c_void_p())
            ret_code = lib.fst_compose_config_new(
                ctypes.c_size_t(compose_filter.value),
                ctypes.c_bool(connect),
                ctypes.byref(config),
            )
            err_msg = "Error creating ComposeConfig"
            check_ffi_error(ret_code, err_msg)
            self.ptr = config
        else:
            raise ValueError("Could not create ComposeConfig")


def compose(fst: VectorFst, other_fst: VectorFst) -> VectorFst:
    """
    compose(fst, other_fst)
    compute the composition of two FSTs.
    :param fst: Fst
    :param other_fst: Fst
    :return: Fst
    """

    composition = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_compose(fst.ptr, other_fst.ptr, ctypes.byref(composition))
    err_msg = "Error Composing FSTs"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=composition)


def compose_with_config(
    fst: VectorFst, other_fst: VectorFst, config: ComposeConfig
) -> VectorFst:
    """
    compose(fst, other_fst)
    compute the composition of two FSTs.
    :param fst: Fst
    :param other_fst: Fst
    :param config: ComposeConfig
    :return: Fst
    """

    composition = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_compose_with_config(
        fst.ptr, other_fst.ptr, config.ptr, ctypes.byref(composition)
    )
    err_msg = "Error Composing FSTs"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=composition)
