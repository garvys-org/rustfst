#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from rustfst_tests_data.fst_test_data import FstTestData, weight_one


class FstTestData009(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst(arc_type="standard")
        s0 = fst.add_state()
        s1 = fst.add_state()
        s2 = fst.add_state()
        s3 = fst.add_state()
        s4 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s4, 0.7)

        fst.add_arc(s0, p.Arc(12, 12, p.Weight(fst.weight_type(), 0.3), s1))
        fst.add_arc(s1, p.Arc(13, 13, p.Weight(fst.weight_type(), 0.4), s3))

        fst.add_arc(s0, p.Arc(12, 12, p.Weight(fst.weight_type(), 0.3), s2))
        fst.add_arc(s2, p.Arc(13, 13, p.Weight(fst.weight_type(), 0.4), s3))
        fst.add_arc(s2, p.Arc(15, 15, p.Weight(fst.weight_type(), 0.1), s4))
        fst.add_arc(s2, p.Arc(16, 16, p.Weight(fst.weight_type(), 0.1), s2))
        fst.add_arc(s0, p.Arc(17, 17, p.Weight(fst.weight_type(), 0.15), s3))

        fst.add_arc(s3, p.Arc(14, 14, p.Weight(fst.weight_type(), 0.6), s4))

        return fst


if __name__ == "__main__":
    FstTestData009("fst_009", os.path.dirname(__file__)).compute_data()
