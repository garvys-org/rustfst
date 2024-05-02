from rustfst import VectorFst, Tr

from rustfst.algorithms.minimize import MinimizeConfig


def test_minimize_fst():
    fst = VectorFst()
    s0 = fst.add_state()
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s0)
    fst.set_final(s1, 0.0)
    fst.set_final(s2, 0.0)

    fst.add_tr(s0, Tr(1, 1, 0.0, s1))
    fst.add_tr(s0, Tr(2, 2, 0.0, s2))

    fst.minimize()

    assert fst.num_states() == 2
    assert fst.num_trs(s0) == 2


def test_minimize_fst_with_config():
    fst = VectorFst()
    s0 = fst.add_state()
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s0)
    fst.set_final(s1, 0.0)
    fst.set_final(s2, 0.0)

    fst.add_tr(s0, Tr(1, 1, 0.0, s1))
    fst.add_tr(s0, Tr(1, 1, 0.0, s2))

    fst.minimize(
        MinimizeConfig(
            allow_nondet=True,
        )
    )

    assert fst.num_states() == 2
    assert fst.num_trs(s0) == 1
