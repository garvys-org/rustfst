#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from fst_test_data.fst_test_data import FstTestData, weight_one


class FstTestData004(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst()
        s0 = fst.add_state()
        s1 = fst.add_state()
        fst.add_state()
        s3 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s1, 0.7)

        fst.add_arc(s0, p.Arc(12, 25, 0.3, s1))
        fst.add_arc(s0, p.Arc(10, 26, 0.4, s1))
        fst.add_arc(s1, p.Arc(4, 5, 0.1, s3))

        return fst


if __name__ == "__main__":
    FstTestData004("fst_004", os.path.dirname(__file__)).compute_data()
