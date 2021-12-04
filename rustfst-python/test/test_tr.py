from rustfst import Tr


def test_tr():
    b = Tr(1, 1, 1.0, 2)

    assert b.ilabel == 1
    assert b.olabel == 1
    assert b.weight == 1.0
    assert b.next_state == 2

    b.ilabel = 2
    b.olabel = 3
    b.weight = 4.0
    b.next_state = 5

    assert b.ilabel == 2
    assert b.olabel == 3
    assert b.weight == 4.0
    assert b.next_state == 5
