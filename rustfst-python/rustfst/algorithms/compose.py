from __future__ import annotations
import ctypes
from typing import Optional, List

from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


from enum import Enum


class MatcherRewriteMode(Enum):
    AUTO = 0
    ALWAYS = 1
    NEVER = 2


class CIntArray(ctypes.Structure):
    _fields_ = [("data", ctypes.POINTER(ctypes.c_uint32)), ("size", ctypes.c_uint32)]


class MatcherConfig:
    def __init__(
        self,
        sigma_label: int,
        rewrite_mode: MatcherRewriteMode = MatcherRewriteMode.AUTO,
        sigma_allowed_matches: Optional[List[int]] = None,
    ):
        array = []
        if sigma_allowed_matches is not None:
            array = sigma_allowed_matches

        arr = CIntArray()
        arr.size = ctypes.c_uint32(len(array))
        arr.data = (ctypes.c_uint32 * len(array))(*array)

        config = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_matcher_config_new(
            ctypes.c_size_t(sigma_label),
            ctypes.c_size_t(rewrite_mode.value),
            arr,
            ctypes.byref(config),
        )
        err_msg = "Error creating MatcherConfig"
        check_ffi_error(ret_code, err_msg)
        self.ptr = config

    def __del__(self):
        lib.fst_matcher_config_destroy(self.ptr)


class ComposeFilter(Enum):
    AUTOFILTER = 0
    NULLFILTER = 1
    TRIVIALFILTER = 2
    SEQUENCEFILTER = 3
    ALTSEQUENCEFILTER = 4
    MATCHFILTER = 5
    NOMATCHFILTER = 6


class ComposeConfig:
    """
    Configuration for compose operation.

    Args:
      compose_filter: Filter which determines allowable matches during
                      composition operation.
      connect: Connect the resulting FST after composition.
      matcher1_config: Matcher configuration for left-hand FST.
      matcher2_config: Matcher configuration for right-hand FST.
    """

    def __init__(
        self,
        compose_filter: ComposeFilter = ComposeFilter.AUTOFILTER,
        connect: bool = True,
        matcher1_config: Optional[MatcherConfig] = None,
        matcher2_config: Optional[MatcherConfig] = None,
    ):
        config = ctypes.pointer(ctypes.c_void_p())

        m1_ptr = None
        if matcher1_config is not None:
            m1_ptr = matcher1_config.ptr
        m2_ptr = None

        if matcher2_config is not None:
            m2_ptr = matcher2_config.ptr

        ret_code = lib.fst_compose_config_new(
            ctypes.c_size_t(compose_filter.value),
            ctypes.c_bool(connect),
            m1_ptr,
            m2_ptr,
            ctypes.byref(config),
        )
        err_msg = "Error creating ComposeConfig"
        check_ffi_error(ret_code, err_msg)
        self.ptr = config

    def __del__(self):
        lib.fst_compose_config_destroy(self.ptr)


def compose(fst: VectorFst, other_fst: VectorFst) -> VectorFst:
    """
    Compute the composition of two FSTs.
    Args:
        fst: Left fst.
        other_fst: Right fst.
    Returns:
        Resulting fst.
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
    Compute the composition of two FSTs parametrized with a config.
    Args:
        fst: Left fst.
        other_fst: Right fst.
        config: Config parameters of the composition.
    Returns:
        Resulting fst.
    """

    composition = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_compose_with_config(
        fst.ptr, other_fst.ptr, config.ptr, ctypes.byref(composition)
    )
    err_msg = "Error Composing FSTs"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=composition)
