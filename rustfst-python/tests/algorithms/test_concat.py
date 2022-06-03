from rustfst import VectorFst, Tr
from rustfst.algorithms.concat import concat_list


def test_concat_fst():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2, 0.2)

    tr1_1 = Tr(1, 2, 1.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(3, 4, 2.5, s2)
    fst1.add_tr(s2, tr1_2)

    # FST 2
    fst2 = VectorFst()

    s1 = fst2.add_state()
    s2 = fst2.add_state()

    fst2.set_start(s1)
    fst2.set_final(s2, 1.5)

    tr2_1 = Tr(1, 2, 3.0, s1)
    fst2.add_tr(s1, tr2_1)

    tr2_2 = Tr(4, 5, 2.0, s2)
    fst2.add_tr(s1, tr2_2)

    # Expected FST
    expected_fst = VectorFst()
    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()
    s4 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s4, 1.5)

    tr3_1 = Tr(1, 2, 1.0, s2)
    expected_fst.add_tr(s1, tr3_1)

    tr3_2 = Tr(3, 4, 2.5, s2)
    expected_fst.add_tr(s2, tr3_2)

    tr3_3 = Tr(0, 0, 0.2, s3)
    expected_fst.add_tr(s2, tr3_3)

    tr3_4 = Tr(1, 2, 3.0, s3)
    expected_fst.add_tr(s3, tr3_4)

    tr3_5 = Tr(4, 5, 2.0, s4)
    expected_fst.add_tr(s3, tr3_5)

    fst3 = fst1.concat(fst2)

    assert fst3 == expected_fst


def test_concat_list():
    concat_list([VectorFst(), VectorFst(), VectorFst()])
