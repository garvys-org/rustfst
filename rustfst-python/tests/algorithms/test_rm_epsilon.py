from rustfst import VectorFst, Tr

# from rustfst import DrawingConfig


def test_rm_epsilon():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()
    s4 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s4, 1.0)

    tr1_1 = Tr(0, 0, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(1, 0, 2.0, s3)
    fst1.add_tr(s2, tr1_2)

    tr1_3 = Tr(0, 2, 3.0, s3)
    fst1.add_tr(s2, tr1_3)

    tr1_4 = Tr(0, 0, 4.0, s3)
    fst1.add_tr(s2, tr1_4)

    tr1_5 = Tr(0, 0, 5.0, s3)
    fst1.add_tr(s3, tr1_5)

    tr1_6 = Tr(0, 0, 5.0, s4)
    fst1.add_tr(s3, tr1_6)

    # d = DrawingConfig()
    # fst1.draw("rm_eps_in.dot", None, None, d)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s1, 11.0)
    expected_fst.set_final(s2, 6.0)

    tr1_1 = Tr(0, 2, 4.0, s2)
    expected_fst.add_tr(s1, tr1_1)

    tr1_2 = Tr(1, 0, 3.0, s2)
    expected_fst.add_tr(s1, tr1_2)
    # expected_fst.draw("rm_eps_e.dot", None, None, d)

    fst1.rm_epsilon()
    # fst1.draw("rm_eps_out.dot", None, None, d)
    assert expected_fst == fst1
