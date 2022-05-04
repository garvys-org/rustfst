from rustfst.algorithms import transducer
from rustfst import DrawingConfig, SymbolTable


def test_transducer():
    symt = SymbolTable()
    symt.add_symbol("hello")
    symt.add_symbol("world")
    symt.add_symbol("coucou")
    symt.add_symbol("monde")

    f = transducer("hello world", "coucou monde", symt, symt)
    d = DrawingConfig()
    f.draw("acceptor.dot", None, None, d)
