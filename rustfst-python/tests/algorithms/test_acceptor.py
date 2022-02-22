from rustfst.algorithms import acceptor
from rustfst import DrawingConfig, SymbolTable


def test_acceptor():
    symt = SymbolTable()
    symt.add_symbol("hello")
    symt.add_symbol("world")

    f = acceptor("hello world", symt)
    d = DrawingConfig()
    f.draw("acceptor.dot", None, None, d)
