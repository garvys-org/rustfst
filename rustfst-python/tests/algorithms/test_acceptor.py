from rustfst.algorithms import acceptor
from rustfst import VectorFst, Tr, SymbolTable


def test_acceptor():
    symt = SymbolTable()
    symt.add_symbol("hello")
    symt.add_symbol("world")

    f = acceptor("hello world", symt)

    # Expected FST
    expected_fst = VectorFst()

    s1 = expected_fst.add_state()
    s2 = expected_fst.add_state()
    s3 = expected_fst.add_state()

    expected_fst.set_start(s1)
    expected_fst.set_final(s3)

    tr1 = Tr(1, 1, None, s2)
    expected_fst.add_tr(s1, tr1)

    tr2 = Tr(2, 2, None, s3)
    expected_fst.add_tr(s2, tr2)

    assert f == expected_fst
