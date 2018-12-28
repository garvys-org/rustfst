#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from fst_test_data.fst_test_data import FstTestData, weight_one


class FstTestData002(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst()
        s0 = fst.add_state()
        s1 = fst.add_state()
        s2 = fst.add_state()
        s3 = fst.add_state()
        s4 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s3, 0.7)

        fst.add_arc(s0, p.Arc(12, 25, 0.3, s1))
        fst.add_arc(s1, p.Arc(112, 75, 0.1, s2))
        fst.add_arc(s2, p.Arc(124, 75, 0.5, s3))
        fst.add_arc(s3, p.Arc(152, 55, 0.6, s4))

        s5 = fst.add_state()
        s6 = fst.add_state()

        fst.add_arc(s5, p.Arc(12, 25, 0.4, s4))
        fst.add_arc(s5, p.Arc(12, 25, 0.1, s2))

        fst.add_arc(s0, p.Arc(12, 25, 0.3, s6))
        fst.add_arc(s1, p.Arc(12, 25, 0.2, s6))

        return fst


if __name__ == "__main__":
    FstTestData002("fst_002", os.path.dirname(__file__)).compute_data()
