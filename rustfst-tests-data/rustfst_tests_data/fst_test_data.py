#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

import io
import os
import json
import subprocess
import itertools

import pynini as p
from pywrapfst import FstOpError

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
        for map_type in ["identity", "rmweight", "invert", "input_epsilon", "output_epsilon", "quantize"]:
            self.compute_arc_map_identity(map_type)
        self.compute_arc_map_identity("plus", weight=1.5)
        self.compute_arc_map_identity("times", weight=1.5)

        print("ArcSort")
        self.compute_arcsort("ilabel")
        self.compute_arcsort("olabel")

        print("Encode")
        self.compute_encode()

        print("Encode / Decode")
        self.compute_encode_decode()

        print("StateMap : ArcSum")
        self.compute_state_map_arc_sum()

        print("StateMap : ArcUnique")
        self.compute_state_map_arc_unique()

        print("Determinization")
        self.compute_determinization()

        print("Properties")
        self.compute_fst_properties()

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

    def compute_encode(self):
        res = []
        for (l, w) in itertools.product([True, False], repeat=2):
            mapper = p.EncodeMapper(encode_labels=l, encode_weights=w, arc_type=self.raw_fst.arc_type())
            fst_out = self.raw_fst.copy().encode(mapper)
            res.append({
                "encode_labels": l,
                "encode_weights": w,
                "result": fst_out.text()
            })
        self.config["encode"] = res

    def compute_encode_decode(self):
        res = []
        for (l, w) in itertools.product([True, False], repeat=2):
            mapper = p.EncodeMapper(encode_labels=l, encode_weights=w, arc_type=self.raw_fst.arc_type())
            fst_encoded = self.raw_fst.copy().encode(mapper)
            fst_decoded= fst_encoded.decode(mapper)
            res.append({
                "encode_labels": l,
                "encode_weights": w,
                "result": fst_decoded.text()
            })
        self.config["encode_decode"] = res

    def compute_state_map_arc_sum(self):
        fst_out = p.statemap(self.raw_fst.copy(), map_type="arc_sum")
        self.add_data_to_config("state_map_arc_sum", fst_out.text())

    def compute_state_map_arc_unique(self):
        fst_out = p.statemap(self.raw_fst.copy(), map_type="arc_unique")
        self.add_data_to_config("state_map_arc_unique", fst_out.text())

    def compute_determinization(self):
        l_res = []
        for det_type in ["functional", "nonfunctional", "disambiguate"]:
            try:
                fst_out = p.determinize(self.raw_fst.copy(), det_type=det_type)
                res = fst_out.text()
            except FstOpError:
                res = "error"
            l_res.append({
                "det_type": det_type,
                "result": res
            })
        self.config["determinize"] = l_res

    def compute_arcsort(self, sort_type):
        fst_out = self.raw_fst.copy()
        fst_out.arcsort(sort_type=sort_type)
        self.add_data_to_config("arcsort_" + sort_type, fst_out.text())

    def compute_fst_properties(self):
        self.config["fst_properties"] = {}
        fst_out = self.raw_fst.copy()

        a = fst_out.properties(p.TRINARY_PROPERTIES, True)

        def compute_prop(prop_name, prop_pynini):
            self.config["fst_properties"][prop_name] = fst_out.properties(prop_pynini, False) == prop_pynini

        compute_prop("acceptor", p.ACCEPTOR)
        compute_prop("not_acceptor", p.NOT_ACCEPTOR)
        compute_prop("i_deterministic", p.I_DETERMINISTIC)
        compute_prop("not_i_deterministic", p.NON_I_DETERMINISTIC)
        compute_prop("o_deterministic", p.O_DETERMINISTIC)
        compute_prop("not_o_deterministic", p.NON_O_DETERMINISTIC)
        compute_prop("epsilons", p.EPSILONS)
        compute_prop("no_epsilons", p.NO_EPSILONS)
        compute_prop("i_epsilons", p.I_EPSILONS)
        compute_prop("no_i_epsilons", p.NO_I_EPSILONS)
        compute_prop("o_epsilons", p.O_EPSILONS)
        compute_prop("no_o_epsilons", p.NO_O_EPSILONS)
        compute_prop("i_label_sorted", p.I_LABEL_SORTED)
        compute_prop("not_i_label_sorted", p.NOT_I_LABEL_SORTED)
        compute_prop("o_label_sorted", p.O_LABEL_SORTED)
        compute_prop("not_o_label_sorted", p.NOT_O_LABEL_SORTED)
        compute_prop("weighted", p.WEIGHTED)
        compute_prop("unweighted", p.UNWEIGHTED)
        compute_prop("cyclic", p.CYCLIC)
        compute_prop("acyclic", p.ACYCLIC)
        compute_prop("initial_cyclic", p.INITIAL_CYCLIC)
        compute_prop("initial_acyclic", p.INITIAL_ACYCLIC)
        compute_prop("top_sorted", p.TOP_SORTED)
        compute_prop("not_top_sorted", p.NOT_TOP_SORTED)
        compute_prop("accessible", p.ACCESSIBLE)
        compute_prop("not_accessible", p.NOT_ACCESSIBLE)
        compute_prop("coaccessible", p.COACCESSIBLE)
        compute_prop("not_coaccessible", p.NOT_COACCESSIBLE)
        compute_prop("string", p.STRING)
        compute_prop("not_string", p.NOT_STRING)
        compute_prop("weighted_cycles", p.WEIGHTED_CYCLES)
        compute_prop("unweighted_cycles", p.UNWEIGHTED_CYCLES)

        assert len(self.config["fst_properties"]) == 32

