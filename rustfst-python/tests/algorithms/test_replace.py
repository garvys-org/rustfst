from rustfst import VectorFst, Tr


def test_replace():
    # FST 1
    fst1 = VectorFst()

    s1 = fst1.add_state()
    s2 = fst1.add_state()
    s3 = fst1.add_state()
    s4 = fst1.add_state()

    fst1.set_start(s1)
    fst1.set_final(s4)

    # call:call
    tr1_1 = Tr(1, 1, None, s2)
    fst1.add_tr(s1, tr1_1)

    # NAME:NAME
    tr1_2 = Tr(10, 10, None, s3)
    fst1.add_tr(s2, tr1_2)

    # now:now
    tr1_3 = Tr(2, 2, None, s4)
    fst1.add_tr(s3, tr1_3)

    # FST 2
    fst2 = VectorFst()

    s1 = fst2.add_state()
    s2 = fst2.add_state()
    s3 = fst2.add_state()

    fst2.set_start(s1)
    fst2.set_final(s3)

    # FIRST_NAME:FIRST_NAME
    tr2_1 = Tr(11, 11, None, s2)
    fst2.add_tr(s1, tr2_1)

    # LAST_NAME:LAST_NAME
    tr2_2 = Tr(12, 12, None, s3)
    fst2.add_tr(s2, tr2_2)

    # FST 3
    fst3 = VectorFst()

    s1 = fst3.add_state()
    s2 = fst3.add_state()

    fst3.set_start(s1)
    fst3.set_final(s2)

    # david:david
    tr3_1 = Tr(3, 3, None, s2)
    fst3.add_tr(s1, tr3_1)

    # john:john
    tr3_2 = Tr(4, 4, None, s2)
    fst3.add_tr(s1, tr3_2)

    # FST 4
    fst4 = VectorFst()

    s1 = fst4.add_state()
    s2 = fst4.add_state()

    fst4.set_start(s1)
    fst4.set_final(s2)

    # bowie:bowie
    tr4_1 = Tr(5, 5, None, s2)
    fst4.add_tr(s1, tr4_1)

    # williams:williaw
    tr4_2 = Tr(6, 6, None, s2)
    fst4.add_tr(s1, tr4_2)

    # Expected FST
    expected_fst = VectorFst()
    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()
    s4 = expected_fst.add_state()
    s5 = expected_fst.add_state()
    s6 = expected_fst.add_state()
    s7 = expected_fst.add_state()
    s8 = expected_fst.add_state()
    s9 = expected_fst.add_state()
    s10 = expected_fst.add_state()
    s11 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s11)

    # call:call
    tr_1 = Tr(1, 1, None, s2)
    expected_fst.add_tr(s1, tr_1)

    # NAME:<eps>
    tr_2 = Tr(10, 0, None, s3)
    expected_fst.add_tr(s2, tr_2)

    # FIRST_NAME:<eps>
    tr_3 = Tr(11, 0, None, s4)
    expected_fst.add_tr(s3, tr_3)

    # david:david
    tr_4 = Tr(3, 3, None, s5)
    expected_fst.add_tr(s4, tr_4)

    # john:john
    tr_5 = Tr(4, 4, None, s5)
    expected_fst.add_tr(s4, tr_5)

    # <eps>:<eps>
    tr_eps = Tr(0, 0, None, s6)
    expected_fst.add_tr(s5, tr_eps)

    # LAST_NAME:<eps>
    tr_6 = Tr(12, 0, None, s7)
    expected_fst.add_tr(s6, tr_6)

    # bowie:bowie
    tr_7 = Tr(5, 5, None, s8)
    expected_fst.add_tr(s7, tr_7)

    # williams:williaw
    tr_8 = Tr(6, 6, None, s8)
    expected_fst.add_tr(s7, tr_8)

    # <eps>:<eps>
    tr_9 = Tr(0, 0, None, s9)
    expected_fst.add_tr(s8, tr_9)

    # <eps>:<eps>
    tr_10 = Tr(0, 0, None, s10)
    expected_fst.add_tr(s9, tr_10)

    # now:now
    tr_11 = Tr(2, 2, None, s11)
    expected_fst.add_tr(s10, tr_11)

    replaced_fst = fst1.replace(100, [(10, fst2), (11, fst3), (12, fst4)], False)

    assert replaced_fst == expected_fst
