import ctypes

from rustfst.fst.vector_fst import Fst, VectorFst
from rustfst.ffi_utils import (
    lib,
    check_ffi_error,
)


def randgen(
    ifst: Fst,
    npath: int = 1,
    seed: int = 0,
    select: str = "uniform",
    max_length: int = 2147483647,
    weight: bool = False,
    remove_total_weight: bool = False,
) -> VectorFst:
    """
    Randomly generate successful paths in an FST.
    This operation randomly generates a set of successful paths in the input FST.
    This relies on a mechanism for selecting arcs, specified using the `select`
    argument. The default selector, "uniform", randomly selects a transition
    using a uniform distribution. The "log_prob" selector randomly selects a
    transition w.r.t. the weights treated as negative log probabilities after
    normalizing for the total weight leaving the state. In all cases, finality is
    treated as a transition to a super-final state.

    Args:
        ifst: The input FST.
        npath: The number of random paths to generate.
        seed: An optional seed value for random path generation; if zero, the
            current time and process ID is used.
        select: A string matching a known random arc selection type; one of:
            "uniform", "log_prob", "fast_log_prob".
        max_length: The maximum length of each random path.
        weight: Should the output be weighted by path count?
        remove_total_weight: Should the total weight be removed (ignored when
            `weighted` is False)?

    Returns:
        An FST containing one or more random paths.

    Raises:
      ValueError: when something wrong happened.
    """

    if select != "uniform":
        raise ValueError(
            f"Only the uniform distribution is supported for now. Found {select}"
        )

    npath = ctypes.c_size_t(npath)
    seed = ctypes.c_size_t(seed)
    max_length = ctypes.c_size_t(max_length)
    weight = ctypes.c_bool(weight)
    remove_total_weight = ctypes.c_bool(remove_total_weight)
    randgen_fst = ctypes.pointer(ctypes.c_void_p())

    ret_code = lib.fst_randgen(
        ifst.ptr,
        npath,
        seed,
        max_length,
        weight,
        remove_total_weight,
        ctypes.byref(randgen_fst),
    )
    err_msg = "Error during randgen"
    check_ffi_error(ret_code, err_msg)

    return VectorFst(ptr=randgen_fst)
