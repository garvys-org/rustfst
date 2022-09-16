from rustfst import VectorFst, Tr


def test_optimize_fst():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()
    s4 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s4, 0.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(1, 3, 2.0, s3)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(0, 0, 3.0, s4)
    fst1.add_tr(s2, tr1_3)

    tr1_4 = Tr(4, 6, 4.0, s4)
    fst1.add_tr(s2, tr1_4)

    tr1_5 = Tr(7, 8, 5.0, s4)
    fst1.add_tr(s3, tr1_5)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()
    s4 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s2, 0.0)
    expected_fst.set_final(s4, 0.0)

    tr_1 = Tr(1, 2, 4.0, s2)
    expected_fst.add_tr(s1, tr_1)

    tr_2 = Tr(1, 3, 7.0, s3)
    expected_fst.add_tr(s1, tr_2)

    tr_4 = Tr(4, 6, 1.0, s4)
    expected_fst.add_tr(s2, tr_4)

    tr_5 = Tr(7, 8, None, s4)
    expected_fst.add_tr(s3, tr_5)

    fst1.optimize()

    assert fst1 == expected_fst


def test_log_optimize_fst():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()
    s4 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s4, 0.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(1, 3, 2.0, s3)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(0, 0, 3.0, s4)
    fst1.add_tr(s2, tr1_3)

    tr1_4 = Tr(4, 6, 4.0, s4)
    fst1.add_tr(s2, tr1_4)

    tr1_5 = Tr(7, 8, 5.0, s4)
    fst1.add_tr(s3, tr1_5)

    fst1.optimize_in_log()

    assert fst1.num_states() == 4
