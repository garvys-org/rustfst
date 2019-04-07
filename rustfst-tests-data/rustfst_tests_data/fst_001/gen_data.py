#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from rustfst_tests_data.fst_test_data import FstTestData, weight_one


class FstTestData001(FstTestData):
    def get_raw_fst(self):
        fst = p.Fst()
        s0 = fst.add_state()
        s1 = fst.add_state()

        fst.set_start(s0)
        fst.set_final(s1)

        fst.add_arc(s0, p.Arc(12, 25, weight_one(), s1))

        return fst


if __name__ == "__main__":
    FstTestData001("fst_001", os.path.dirname(__file__)).compute_data()
