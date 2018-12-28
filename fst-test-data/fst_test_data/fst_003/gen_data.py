#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from fst_test_data.fst_test_data import FstTestData, weight_one


class FstTestData003(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst()
        s0 = fst.add_state()
        s1 = fst.add_state()
        s2 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s2, 0.7)

        fst.add_arc(s0, p.Arc(12, 25, 0.3, s1))
        fst.add_arc(s0, p.Arc(14, 26, 0.2, s1))
        fst.add_arc(s1, p.Arc(5, 3, 0.1, s2))
        fst.add_arc(s2, p.Arc(6, 7, 0.4, s2))

        return fst


if __name__ == "__main__":
    FstTestData003("fst_003", os.path.dirname(__file__)).compute_data()
