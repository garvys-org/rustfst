#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import io
import os
import json
import subprocess

import pynini as p

from abc import ABCMeta, abstractmethod


def write_text_fst(fst, path_out):
    with io.open(path_out, mode="w", encoding="utf8") as f:
        f.write(unicode(fst.text()))


def dump_json(data, filename):
    with io.open(filename, mode="w", encoding="utf8") as f:
        f.write(unicode(json.dumps(data, ensure_ascii=False, indent=4)))


def weight_one():
    return p.Weight.One(p.Fst().weight_type())


class FstTestData(object):
    __metaclass__ = ABCMeta

    def __init__(self, name, path_dir):
        self.name = name
        self.raw_fst = self.get_raw_fst()
        self.path_dir = path_dir
        self.config = {}

    def write_text_fst(self, fst, filename):
        path_out = os.path.join(self.path_dir, filename)
        write_text_fst(fst, path_out)

    def add_data_to_config(self, operation_name, filename_out, **kwargs):
        self.config[operation_name] = {"result": filename_out}
        self.config[operation_name].update(kwargs)

    def compute_data(self):

        print("Computing data : %s" % self.name)

        self.add_data_to_config("raw", self.raw_fst.text())
        self.config["weight_type"] = self.raw_fst.weight_type()
        self.config["name"] = self.name

        path_dot = os.path.join(self.path_dir, "fst_raw.dot")
        path_ps = os.path.join(self.path_dir, "fst_raw.ps")

        self.raw_fst.draw(path_dot)
        cmd = "dot -Tps %s > %s" % (path_dot, path_ps)
        subprocess.check_call(cmd, shell=True)
        os.remove(path_dot)

        print("Invert")
        self.compute_fst_invert()

        print("Project input")
        self.compute_fst_project_input()

        print("Project output")
        self.compute_fst_project_output()

        print("Reverse")
        self.compute_fst_reverse()

        print("Remove epsilon")
        self.compute_fst_rmepsilon()

        print("Connect")
        self.compute_fst_connect()

        print("Shortest distance")
        self.compute_fst_shortest_distance()

        print("Weight pushing initial")
        self.compute_weight_pushing_initial()

        print("Weight pushing final")
        self.compute_weight_pushing_final()

        print("Arc Map")
        for map_type in ["identity", "rmweight", "invert", "input_epsilon", "output_epsilon"]:
            self.compute_arc_map_identity(map_type)
        self.compute_arc_map_identity("plus", weight=1.5)
        self.compute_arc_map_identity("times", weight=1.5)

        dump_json(self.config, os.path.join(self.path_dir, "metadata.json"))

        print("Done\n")

    @abstractmethod
    def get_raw_fst(self):
        raise NotImplemented

    def compute_fst_project_input(self):
        fst_out = self.raw_fst.copy().project(project_output=False)
        self.add_data_to_config("project_input", fst_out.text())

    def compute_fst_project_output(self):
        fst_out = self.raw_fst.copy().project(project_output=True)
        self.add_data_to_config("project_output", fst_out.text())

    def compute_fst_invert(self):
        fst_out = self.raw_fst.copy().invert()
        self.add_data_to_config("invert", fst_out.text())

    def compute_fst_reverse(self):
        fst_out = p.reverse(self.raw_fst.copy())
        self.add_data_to_config("reverse", fst_out.text())

    def compute_fst_rmepsilon(self):
        fst_out = p.rmepsilon(self.raw_fst.copy(), connect=False)
        self.add_data_to_config("rmepsilon", fst_out.text())

    def compute_fst_connect(self):
        fst_out = self.raw_fst.copy().connect()
        self.add_data_to_config("connect", fst_out.text())

    def compute_fst_shortest_distance(self):
        out = [float(e) for e in p.shortestdistance(self.raw_fst.copy())]
        out = [e if e != float("inf") else None for e in out]
        self.add_data_to_config("shortest_distance", out)

    def compute_weight_pushing_initial(self):
        fst_out = self.raw_fst.copy().push(to_final=False)
        self.add_data_to_config("weight_pushing_initial", fst_out.text())

    def compute_weight_pushing_final(self):
        fst_out = self.raw_fst.copy().push(to_final=True)
        self.add_data_to_config("weight_pushing_final", fst_out.text())

    def compute_arc_map_identity(self, map_type, weight=None):
        fst_out = p.arcmap(self.raw_fst.copy(), map_type=map_type, weight=weight)
        self.add_data_to_config("arc_map_" + map_type, fst_out.text())
