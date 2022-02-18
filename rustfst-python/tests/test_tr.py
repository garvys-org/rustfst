from rustfst import Tr
import pytest


def test_tr():
    a = Tr(1, 1, 1.0, 2)

    assert a.ilabel == 1
    assert a.olabel == 1
    assert pytest.approx(a.weight) == pytest.approx(1.0)
    assert a.next_state == 2

    a.ilabel = 2
    a.olabel = 3
    a.weight = 4.0
    a.next_state = 5

    assert a.ilabel == 2
    assert a.olabel == 3
    assert pytest.approx(a.weight) == pytest.approx(4.0)
    assert a.next_state == 5
