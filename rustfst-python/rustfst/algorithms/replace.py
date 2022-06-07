from __future__ import annotations
from typing import List
import ctypes
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)

from rustfst.fst.vector_fst import VectorFst


class LabelFstPair(ctypes.Structure):
    _fields_ = [
        ("label", ctypes.c_size_t),
        ("fst", ctypes.POINTER(ctypes.c_void_p)),
    ]


def replace(
    root_idx: int, fst_list: List[(int, VectorFst)], epsilon_on_replace: bool
) -> VectorFst:
    """
    Recursively replaces trs in the root FSTs with other FSTs.

    Replace supports replacement of trs in one Fst with another FST. This
    replacement is recursive. Replace takes an array of FST(s). One FST
    represents the root (or topology) machine. The root FST refers to other FSTs
    by recursively replacing trs labeled as non-terminals with the matching
    non-terminal FST. Currently Replace uses the output symbols of the trs to
    determine whether the transition is a non-terminal transition or not. A non-terminal can be
    any label that is not a non-zero terminal label in the output alphabet.

    Note that input argument is a vector of pairs. These correspond to the tuple
    of non-terminal Label and corresponding FST.

    Examples:

    - Root Fst :

    ![replace_in_1](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_1.svg?sanitize=true)

    - Fst for non-terminal #NAME :

    ![replace_in_2](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_2.svg?sanitize=true)

    - Fst for non-terminal #FIRSTNAME :

    ![replace_in_3](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_3.svg?sanitize=true)

    - Fst for non-terminal #LASTNAME :

    ![replace_in_4](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_in_4.svg?sanitize=true)

    - Output :

    ![replace_out](https://raw.githubusercontent.com/Garvys/rustfst-images-doc/master/images/replace_out.svg?sanitize=true)

    Args:
        root_idx:
        fst_list:
        epsilon_on_replace:

    Returns:
        The resulting Fst.

    """
    pairs = [LabelFstPair(label, fst.ptr) for (label, fst) in fst_list]
    pairs_array = (LabelFstPair * len(pairs))(*pairs)
    res_fst = ctypes.pointer(ctypes.c_void_p())
    ret_code = lib.fst_replace(
        ctypes.c_size_t(root_idx),
        ctypes.byref(pairs_array),
        ctypes.c_size_t(len(pairs)),
        ctypes.c_bool(epsilon_on_replace),
        ctypes.byref(res_fst),
    )
    err_msg = "Error performing replace"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=res_fst)
