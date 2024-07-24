from pathlib import Path

from rustfst import VectorFst, Tr, SymbolTable, ConstFst
import pytest
from tempfile import NamedTemporaryFile

from rustfst.weight import weight_one


def test_small_fst():
    fst = VectorFst()

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


def test_final_weight():
    fst = VectorFst()
    s1 = fst.add_state()
    fst.set_final(s1, 2.3)

    assert pytest.approx(fst.final(s1)) == pytest.approx(2.3)


def test_fst_del_states():
    fst = VectorFst()

    # States
    fst.add_state()
    fst.add_state()

    fst.delete_states()

    assert fst.num_states() == 0


def test_fst_states_iterator():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    for idx, state in enumerate(fst.states()):
        assert state == idx


def test_fst_trs_iterator():
    fst = VectorFst()

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

    num_trs = fst.num_trs(s1)
    idx = 0
    for i, tr in enumerate(fst.trs(s1)):
        idx += 1
        assert tr == trs[i]

    assert num_trs == idx


def test_fst_read_write():
    fst = VectorFst()

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

    read_fst = VectorFst.read("/tmp/test.fst")

    assert fst == read_fst


def test_fst_read_write_with_symt():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    tr_1 = Tr(3, 5, 10.0, s2)
    tr_2 = Tr(5, 7, 18.0, s2)
    fst.add_tr(s1, tr_1)
    fst.add_tr(s1, tr_2)

    input_symt = SymbolTable()
    input_symt.add_symbol("a")
    input_symt.add_symbol("b")
    input_symt.add_symbol("c")
    fst.set_input_symbols(input_symt)

    output_symt = SymbolTable()
    fst.set_output_symbols(output_symt)

    fst.write("/tmp/test.fst")

    read_fst = VectorFst.read("/tmp/test.fst")

    assert read_fst.input_symbols().num_symbols() == 4
    assert read_fst.input_symbols().find("a") == 1
    assert read_fst.input_symbols().find("b") == 2
    assert read_fst.input_symbols().find("c") == 3

    assert read_fst.output_symbols().num_symbols() == 1

    assert fst == read_fst


def test_fst_symt():
    fst = VectorFst()
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
    assert fst_in_symbols.find("a") == 1
    assert fst_in_symbols.find("b") == 2
    assert fst_in_symbols.find("c") == 3

    output_symt = SymbolTable()
    fst.set_output_symbols(output_symt)
    fst_out_symbols = fst.output_symbols()

    assert output_symt == fst_out_symbols
    assert fst_out_symbols.num_symbols() == 1


def test_fst_with_symt_mut_fail():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    input_symt = SymbolTable()
    input_symt.add_symbol("a")
    input_symt.add_symbol("b")
    input_symt.add_symbol("c")
    fst.set_input_symbols(input_symt)

    output_symt = SymbolTable()
    fst.set_output_symbols(output_symt)

    with pytest.raises(Exception) as err:
        fst.input_symbols().add_symbol("d")

    assert (
        str(err.value)
        == '`add_symbol` failed: "Could not get a mutable reference to the symbol table"'
    )


def test_fst_print():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    # Check print is not crashing
    print(fst)


def test_fst_to_bytes():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    bytes = fst.to_bytes()

    with NamedTemporaryFile() as f:
        Path(f.name).write_bytes(bytes)
        fst_read = VectorFst.read(f.name)

    assert fst == fst_read


def test_fst_from_bytes():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    with NamedTemporaryFile() as f:
        fst.write(f.name)
        bytes = Path(f.name).read_bytes()
        fst_loaded = VectorFst.from_bytes(bytes)

    assert fst == fst_loaded


def test_fst_io_bytes():
    fst = VectorFst()

    # States
    s1 = fst.add_state()
    s2 = fst.add_state()

    fst.set_start(s1)
    fst.set_final(s2)

    fst_loaded = VectorFst.from_bytes(fst.to_bytes())

    assert fst_loaded == fst


def test_fst_unset_final():
    fst = VectorFst()

    s = fst.add_state()

    assert not fst.is_final(s)
    fst.set_final(s)
    assert fst.is_final(s)
    fst.unset_final(s)
    assert not fst.is_final(s)


def test_fst_relabel_tables():
    fst = VectorFst()
    s1 = fst.add_state()
    s2 = fst.add_state()
    fst.add_tr(s1, Tr(1, 2, weight_one(), s2))
    fst.set_start(s1)
    fst.set_final(s2)

    old_isymt = SymbolTable.from_symbols(["a", "b"])
    new_isymt = SymbolTable.from_symbols(["b", "a"])

    old_osymt = SymbolTable.from_symbols(["aa", "bb"])
    new_osymt = SymbolTable.from_symbols(["bb", "aa"])

    fst_ref = VectorFst()
    s1 = fst_ref.add_state()
    s2 = fst_ref.add_state()
    fst_ref.add_tr(s1, Tr(2, 1, weight_one(), s2))
    fst_ref.set_start(s1)
    fst_ref.set_final(s2)

    fst_1 = fst.copy()
    fst_1.relabel_tables(
        old_isymbols=old_isymt,
        new_isymbols=new_isymt,
        attach_new_isymbols=True,
        old_osymbols=old_osymt,
        new_osymbols=new_osymt,
        attach_new_osymbols=True,
    )
    assert fst_1 == fst_ref
    assert fst_1.input_symbols() == new_isymt
    assert fst_1.output_symbols() == new_osymt

    fst_2 = fst.copy()
    fst_2.relabel_tables(
        old_isymbols=old_isymt,
        new_isymbols=new_isymt,
        attach_new_isymbols=False,
        old_osymbols=old_osymt,
        new_osymbols=new_osymt,
        attach_new_osymbols=False,
    )
    assert fst_2 == fst_ref
    assert fst_2.input_symbols() is None
    assert fst_2.output_symbols() is None

    fst_3 = fst.copy()
    fst_3.set_input_symbols(old_isymt)
    fst_3.set_output_symbols(old_osymt)
    fst_3.relabel_tables(
        new_isymbols=new_isymt,
        new_osymbols=new_osymt,
    )
    assert fst_3 == fst_ref
    assert fst_3.input_symbols() == new_isymt
    assert fst_3.output_symbols() == new_osymt


def test_const_fst_from_vector_fst():
    fst = VectorFst()
    s1 = fst.add_state()
    s2 = fst.add_state()
    fst.add_tr(s1, Tr(1, 2, weight_one(), s2))
    fst.set_start(s1)
    fst.set_final(s2)

    const_fst = ConstFst.from_vector_fst(fst)

    assert const_fst.num_trs(const_fst.start()) == 1
