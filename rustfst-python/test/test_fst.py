from rustfst import Fst, Tr

def test_small_fst():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)

    # Trs
    tr_1 = Tr(3, 5, 10.0, s2)
    fst.add_tr(s1, tr_1)

    assert fst.num_trs(s1) == 1

    tr_2 = Tr(5, 7, 18.0, s2)
    fst.add_tr(s1, tr_2)
    assert fst.num_trs(s1) == 2

def test_fst_states_iterator():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    for idx, state in enumerate(fst.states()):
        assert state == idx