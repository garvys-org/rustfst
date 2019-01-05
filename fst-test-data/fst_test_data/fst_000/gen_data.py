#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import os
import pynini as p
from fst_test_data.fst_test_data import FstTestData, weight_one


class FstTestData000(FstTestData):
    def get_raw_fst(self):

        return p.Fst()


if __name__ == "__main__":
    FstTestData000("fst_000", os.path.dirname(__file__)).compute_data()
