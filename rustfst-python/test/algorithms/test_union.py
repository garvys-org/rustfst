from rustfst import VectorFst, Tr
from rustfst import DrawingConfig


def test_union():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s3)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 4, 2.0, s2)
    fst1.add_tr(s1, tr1_2)

    tr1_3 = Tr(4, 5, 3.0, s3)
    fst1.add_tr(s2, tr1_3)

    d = DrawingConfig()
    # fst1.draw("union_1.dot", None, None, d)

    # FST 2
    fst2 = VectorFst()

    s1 = fst2.add_state()
    s2 = fst2.add_state()

    fst2.set_start(s1)
    fst2.set_final(s2, 0.2)

    tr2_1 = Tr(1, 2, 1.0, s2)
    fst2.add_tr(s1, tr2_1)

    tr2_2 = Tr(3, 4, 2.5, s2)
    fst2.add_tr(s2, tr2_2)

    # fst2.draw("concat_2.dot", None, None, d)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()
    s4 = expected_fst.add_state()
    s5 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s3)
    expected_fst.set_final(s5, 0.2)

    tr_1 = Tr(1, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr_1)

    tr_2 = Tr(3, 4, 2.0, s2)
    expected_fst.add_tr(s1, tr_2)

    tr_3 = Tr(4, 5, 3.0, s3)
    expected_fst.add_tr(s2, tr_3)

    tr_4 = Tr(0, 0, None, s4)
    expected_fst.add_tr(s1, tr_4)

    tr_5 = Tr(1, 2, 1.0, s5)
    expected_fst.add_tr(s4, tr_5)

    tr_6 = Tr(3, 4, 2.5, s5)
    expected_fst.add_tr(s5, tr_6)
    expected_fst.draw("union_expected.dot", None, None, d)

    union_fst = fst1.union(fst2)
    union_fst.draw("union_res.dot", None, None, d)

    assert union_fst == expected_fst
