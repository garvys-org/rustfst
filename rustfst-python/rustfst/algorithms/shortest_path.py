from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst

KSHORTESTDELTA = 1e-6


class ShortestPathConfig:
    def __init__(self, nshortest: int = 1, unique: bool = False, delta=None):
        if delta is None:
            delta = KSHORTESTDELTA
        config = ctypes.pointer(ctypes.c_void_p())
        ret_code = lib.fst_shortest_path_config_new(
            ctypes.c_float(delta),
            ctypes.c_size_t(nshortest),
            ctypes.c_bool(unique),
            ctypes.byref(config),
        )
        err_msg = "Error creating ShortestPathConfig"
        check_ffi_error(ret_code, err_msg)
        self.ptr = config


def shortestpath(fst: VectorFst) -> VectorFst:
    """
    shortestpath(fst)
    construct a FST containing the shortest path of the input FST
    :param fst: Fst
    :return: Fst
    """

    shortest_path = ctypes.c_void_p()
    ret_code = lib.fst_shortest_path(fst.ptr, ctypes.byref(shortest_path))
    err_msg = "Error computing shortest path"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=shortest_path)


def shortestpath_with_config(fst: VectorFst, config: ShortestPathConfig) -> VectorFst:
    """
    shortestpath(fst,config)
    construct a FST containing the n-shortest path(s) in the input FST
    :param fst: Fst
    :param config: ShortestPathConfig
    :return: Fst
    """

    shortest_path = ctypes.c_void_p()

    ret_code = lib.fst_shortest_path_with_config(
        fst.ptr, config.ptr, ctypes.byref(shortest_path)
    )
    err_msg = "Error computing shortest path"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=shortest_path)
