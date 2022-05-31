from rustfst import VectorFst, Tr


def test_top_sort():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s2)
    fst1.set_final(s1, 0.0)

    tr2_1 = Tr(1, 2, 1.0, s1)
    fst1.add_tr(s2, tr2_1)

    assert fst1.start() == s2
    fst1.top_sort()
    assert fst1.start() == s1
