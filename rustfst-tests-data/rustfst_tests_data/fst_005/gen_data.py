#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from rustfst_tests_data.fst_test_data import FstTestData, weight_one


class FstTestData005(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst(arc_type="log")
        s0 = fst.add_state()
        s1 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s1, 0.7)

        fst.add_arc(s0, p.Arc(12, 25, p.Weight(fst.weight_type(), 0.3), s1))
        fst.add_arc(s0, p.Arc(12, 25, p.Weight(fst.weight_type(), 0.4), s1))
        fst.add_arc(s0, p.Arc(12, 25, p.Weight(fst.weight_type(), 0.1), s1))
        fst.add_arc(s0, p.Arc(12, 26, p.Weight(fst.weight_type(), 0.7), s1))
        fst.add_arc(s0, p.Arc(12, 25, p.Weight(fst.weight_type(), 0.5), s1))
        fst.add_arc(s0, p.Arc(12, 26, p.Weight(fst.weight_type(), 0.2), s1))

        return fst


if __name__ == "__main__":
    FstTestData005("fst_005", os.path.dirname(__file__)).compute_data()
