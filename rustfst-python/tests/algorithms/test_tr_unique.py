from rustfst import VectorFst, Tr


def test_tr_unique_1():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2, 0.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(1, 2, 2.0, s2)
    fst1.add_tr(s1, tr1_3)

    tr1_4 = Tr(2, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_4)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s2, 0.0)

    tr_1 = Tr(1, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr_1)

    tr_2 = Tr(1, 2, 2.0, s2)
    expected_fst.add_tr(s1, tr_2)

    tr_4 = Tr(2, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr_4)

    fst1.tr_unique()

    assert fst1 == expected_fst


# def test_tr_unique_2():
#    # FST 1
#    fst1 = VectorFst()
#
#    s1 = fst1.add_state()
#    s2 = fst1.add_state()
#
#    fst1.set_start(s1)
#    fst1.set_final(s2, 0.0)
#
#    tr1_1 = Tr(1, 2, 1.0, s2)
#    fst1.add_tr(s1, tr1_1)
#
#    tr1_2 = Tr(1, 2, 2.0, s2)
#    fst1.add_tr(s1, tr1_2)
#
#    tr1_3 = Tr(1, 2, 1.0, s2)
#    fst1.add_tr(s1, tr1_3)
#
#    tr1_4 = Tr(2, 2, 1.0, s2)
#    fst1.add_tr(s1, tr1_4)
#
#    # Expected FST
#    expected_fst = VectorFst()
#
#    s1 = expected_fst.add_state()
#    s2 = expected_fst.add_state()
#
#    expected_fst.set_start(s1)
#    expected_fst.set_final(s2, 0.0)
#
#    tr_1 = Tr(1, 2, 1.0, s2)
#    expected_fst.add_tr(s1, tr_1)
#
#    tr_2 = Tr(1, 2, 2.0, s2)
#    expected_fst.add_tr(s1, tr_2)
#
#    tr_4 = Tr(2, 2, 1.0, s2)
#    expected_fst.add_tr(s1, tr_4)
#
#    fst1.tr_unique()
#
#    assert fst1 == expected_fst
