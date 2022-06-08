from rustfst import SymbolTable
from rustfst import VectorFst, Tr


def test_string_paths_iterator():

    fst = VectorFst()
    s1 = fst.add_state()
    s2 = fst.add_state()
    fst.set_start(s1)
    fst.set_final(s2, 2.0)
    fst.add_tr(s1, Tr(1, 2, 2.0, s2))
    fst.add_tr(s1, Tr(2, 3, 3.0, s2))

    symt = SymbolTable()
    symt.add_symbol("a")
    symt.add_symbol("b")
    symt.add_symbol("c")
    fst.set_input_symbols(symt)
    fst.set_output_symbols(symt)

    string_paths_it = fst.string_paths()

    assert not string_paths_it.done()

    v1 = next(string_paths_it)
    assert v1.weight() == 4.0
    assert v1.istring() == "a"
    assert v1.ostring() == "b"
    assert not string_paths_it.done()

    v2 = next(string_paths_it)
    assert v2.weight() == 5.0
    assert v2.istring() == "b"
    assert v2.ostring() == "c"

    assert string_paths_it.done()
