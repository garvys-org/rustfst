from rustfst import VectorFst, Tr


def test_invert():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s3, 1.0)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 4, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(5, 6, 1.5, s2)
    fst1.add_tr(s2, tr1_3)

    tr1_4 = Tr(3, 5, 1.0, s3)
    fst1.add_tr(s2, tr1_4)

    fst1.invert()
    fst1.draw("wtf.dot")

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s3, 1.0)

    tr1_1 = Tr(2, 1, 1.0, s2)
    expected_fst.add_tr(s1, tr1_1)

    tr1_2 = Tr(4, 3, 2.0, s2)
    expected_fst.add_tr(s1, tr1_2)

    tr1_3 = Tr(6, 5, 1.5, s2)
    expected_fst.add_tr(s2, tr1_3)

    tr1_4 = Tr(5, 3, 1.0, s3)
    expected_fst.add_tr(s2, tr1_4)

    expected_fst.draw("omg.dot")
    assert expected_fst == fst1
