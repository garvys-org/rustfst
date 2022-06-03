from rustfst import VectorFst, Tr
from rustfst.weight import weight_one
from rustfst.algorithms.randgen import randgen


def test_randgen():
    fst = VectorFst()
    s0 = fst.add_state()
    s1 = fst.add_state()

    fst.set_start(s0)
    fst.set_final(s1)

    fst.add_tr(s0, Tr(2, 2, weight_one(), s1))
    fst.add_tr(s0, Tr(3, 3, weight_one(), s1))

    res = randgen(ifst=fst, seed=33)

    assert res.num_states() == 2
    for tr in fst.trs(fst.start()):
        assert tr.ilabel in {2, 3}
        assert tr.olabel in {2, 3}
