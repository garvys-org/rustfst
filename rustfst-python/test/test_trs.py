from rustfst import Trs, Tr


def test_trs():
    a = Tr(1, 1, 1.0, 2)
    b = Tr(2, 2, 2.0, 3)
    c = Tr(3, 3, 3.0, 4)

    trs = Trs()
    trs.push(a)
    trs.push(b)
    trs.push(c)

    assert trs.len() == 3

    assert trs.remove(2) == c
    assert trs.remove(1) == b

    assert trs.len() == 1
