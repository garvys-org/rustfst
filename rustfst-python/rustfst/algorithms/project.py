from __future__ import annotations
import ctypes
from rustfst.utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst
from enum import Enum


class ProjectType(Enum):
    PROJECT_INPUT = 0
    PROJECT_OUTPUT = 1


def project(fst: VectorFst, proj_type: ProjectType) -> VectorFst:
    """
    project(fst)
    convert a FST to an acceptor using input or output labels.
    :param fst: Fst
    :return: Fst
    """
    config = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_project_type_new(
        ctypes.c_size_t(proj_type.value),
        ctypes.byref(config),
    )
    err_msg = "Error creating ProjectType"
    check_ffi_error(ret_code, err_msg)

    ret_code = lib.fst_project(fst.ptr, config)
    err_msg = "Error during projection"
    check_ffi_error(ret_code, err_msg)

    return fst
