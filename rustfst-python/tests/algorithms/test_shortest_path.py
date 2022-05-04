from rustfst import VectorFst, Tr
from rustfst.algorithms.shortest_path import ShortestPathConfig


def test_shortest_path():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()
    s4 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s4, 2.0)

    tr1_1 = Tr(1, 1, 3.0, s2)
    fst1.add_tr(s1, tr1_1)

    tr1_2 = Tr(2, 2, 2.0, s2)
    fst1.add_tr(s2, tr1_2)

    tr1_3 = Tr(3, 3, 4.0, s4)
    fst1.add_tr(s2, tr1_3)

    tr1_4 = Tr(4, 4, 5.0, s3)
    fst1.add_tr(s1, tr1_4)

    tr1_5 = Tr(5, 5, 4.0, s4)
    fst1.add_tr(s3, tr1_5)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()

    expected_fst.set_start(s3)
    expected_fst.set_final(s1, 2.0)

    tr1_1 = Tr(1, 1, 3.0, s2)
    expected_fst.add_tr(s3, tr1_1)

    tr1_2 = Tr(3, 3, 4.0, s1)
    expected_fst.add_tr(s2, tr1_2)

    config = ShortestPathConfig(1, True)
    shortes_path = fst1.shortest_path(config)

    assert shortes_path == expected_fst
