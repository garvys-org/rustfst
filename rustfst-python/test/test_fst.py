from rustfst import Fst, Tr, SymbolTable
import pytest


def test_small_fst():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    assert s1 == 0
    assert s2 == 1

    fst.set_start(s1)
    fst.set_final(s2)
    assert fst.start() == s1
    assert fst.is_final(s2)
    assert pytest.approx(fst.final(s2)) == pytest.approx(0.0)

    # Trs
    tr_1 = Tr(3, 5, 10.0, s2)
    fst.add_tr(s1, tr_1)

    assert fst.num_trs(s1) == 1

    tr_2 = Tr(5, 7, 18.0, s2)
    fst.add_tr(s1, tr_2)
    assert fst.num_trs(s1) == 2


def test_fst_del_states():
    fst = Fst()

    # States
    fst.add_state()
    fst.add_state()

    fst.delete_states()

    assert fst.num_states() == 0


def test_fst_states_iterator():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    for idx, state in enumerate(fst.states()):
        assert state == idx


def test_fst_trs_iterator():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    tr_1 = Tr(3, 5, 10.0, s2)
    tr_2 = Tr(5, 7, 18.0, s2)
    fst.add_tr(s1, tr_1)
    fst.add_tr(s1, tr_2)

    trs = [tr_1, tr_2]

    for i, tr in enumerate(fst.trs(s1)):
        assert tr == trs[i]


def test_fst_read_write():
    fst = Fst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    tr_1 = Tr(3, 5, 10.0, s2)
    tr_2 = Tr(5, 7, 18.0, s2)
    fst.add_tr(s1, tr_1)
    fst.add_tr(s1, tr_2)

    fst.write("/tmp/test.fst")

    read_fst = Fst.read("/tmp/test.fst")

    assert fst == read_fst


def test_fst_symt():
    fst = Fst()
    s1 = fst.add_state()
    s2 = fst.add_state()
    fst.set_start(s1)
    fst.set_final(s2, 1.0)

    tr_1 = Tr(1, 0, 10.0, s2)
    tr_2 = Tr(2, 0, 1.0, s1)
    tr_3 = Tr(3, 0, 1.0, s2)
    fst.add_tr(s1, tr_1)
    fst.add_tr(s2, tr_2)
    fst.add_tr(s2, tr_3)

    input_symt = SymbolTable()
    input_symt.add_symbol("a")
    input_symt.add_symbol("b")
    input_symt.add_symbol("c")

    fst.set_input_symbols(input_symt)
    fst_in_symbols = fst.input_symbols()

    assert input_symt == fst_in_symbols
    assert fst_in_symbols.num_symbols() == 4

    output_symt = SymbolTable()
    fst.set_output_symbols(output_symt)
    fst_out_symbols = fst.output_symbols()

    assert output_symt == fst_out_symbols
    assert fst_out_symbols.num_symbols() == 1
