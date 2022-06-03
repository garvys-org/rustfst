from __future__ import annotations
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst
from enum import Enum


class ProjectType(Enum):
    """
    Different types of labels projection in a Fst.
    """

    PROJECT_INPUT = 0
    """
    Input projection : output labels are replaced with input ones.
    """
    PROJECT_OUTPUT = 1
    """
    Output projection : input labels are replaced with output ones.
    """


def project(fst: VectorFst, proj_type: ProjectType) -> VectorFst:
    """
    Convert a Fst to an acceptor using input or output labels.
    Args:
        fst: Fst on which to apply the algorithm.
        proj_type: Whether to replace input labels or output labels.
    Returns:
        The resulting Fst.
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


if __name__ == "__main__":
    import doctest

    doctest.testmod()
