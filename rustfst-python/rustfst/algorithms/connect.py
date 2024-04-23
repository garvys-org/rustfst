from __future__ import annotations
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


def connect(fst: VectorFst) -> VectorFst:
    """
    This operation trims an Fst, removing states and trs that are not on successful paths.

    Examples :

    - Input :

    ![connect_in](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_in.svg?sanitize=true)

    - Output :

    ![connect_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/connect_out.svg?sanitize=true)

    Returns :
        self

    """

    ret_code = lib.fst_connect(fst.ptr)
    err_msg = "Error during connect"
    check_ffi_error(ret_code, err_msg)

    return fst
