from rustfst import VectorFst, Tr


def test_isomorphic_1():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2)

    tr1_1 = Tr(12, 25, None, s2)
    fst1.add_tr(s1, tr1_1)

    # FST 2
    fst2 = fst1.copy()

    assert fst1.isomorphic(fst2)

    fst2.add_tr(s1, Tr(1, 2, None, s2))

    assert not fst1.isomorphic(fst2)


def test_isomorphic_2():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s2)

    tr1_1 = Tr(12, 25, None, s2)
    fst1.add_tr(s1, tr1_1)

    # FST 2
    fst2 = VectorFst()
    s1 = fst2.add_state()
    s2 = fst2.add_state()

    fst2.set_start(s2)
    fst2.set_final(s1)

    tr1_1 = Tr(12, 25, None, s1)
    fst2.add_tr(s2, tr1_1)

    assert fst1.isomorphic(fst2)
