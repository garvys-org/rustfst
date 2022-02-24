from rustfst import VectorFst, Tr


def test_tr_sort_ilabel():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2, 0.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 3, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(1, 5, 3.0, s2)
    fst1.add_tr(s1, tr1_3)

    tr1_4 = Tr(2, 6, 4.0, s2)
    fst1.add_tr(s1, tr1_4)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s2, 0.0)

    tr_1 = Tr(1, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr_1)

    tr_2 = Tr(1, 5, 3.0, s2)
    expected_fst.add_tr(s1, tr_2)

    tr_4 = Tr(2, 6, 4.0, s2)
    expected_fst.add_tr(s1, tr_4)

    tr_5 = Tr(3, 3, 2.0, s2)
    expected_fst.add_tr(s1, tr_5)

    fst1.tr_sort()

    assert fst1 == expected_fst


def test_tr_sort_olabel():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2, 0.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 3, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(1, 5, 3.0, s2)
    fst1.add_tr(s1, tr1_3)

    tr1_4 = Tr(2, 6, 4.0, s2)
    fst1.add_tr(s1, tr1_4)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s2, 0.0)

    tr_1 = Tr(1, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr_1)

    tr_2 = Tr(3, 3, 2.0, s2)
    expected_fst.add_tr(s1, tr_2)

    tr_4 = Tr(1, 5, 3.0, s2)
    expected_fst.add_tr(s1, tr_4)

    tr_5 = Tr(2, 6, 4.0, s2)
    expected_fst.add_tr(s1, tr_5)

    fst1.tr_sort(ilabel_cmp=False)

    assert fst1 == expected_fst
