from rustfst_python_bench.algorithms.arcsort import ArcSortAlgorithm
from rustfst_python_bench.algorithms.connect import ConnectAlgorithm
from rustfst_python_bench.algorithms.invert import InvertAlgorithm
from rustfst_python_bench.algorithms.project import ProjectAlgorithm
from rustfst_python_bench.algorithms.minimize import MinimizeAlgorithm
from rustfst_python_bench.algorithms.push import PushAlgorithm
from rustfst_python_bench.algorithms.reverse import ReverseAlgorithm
from rustfst_python_bench.algorithms.rm_final_epsilon import RmFinalEpsilonAlgorithm
from rustfst_python_bench.algorithms.shortestpath import ShortestPathAlgorithm
from rustfst_python_bench.algorithms.map import MapAlgorithm
from rustfst_python_bench.algorithms.compose import ComposeAlgorithm


class SupportedAlgorithms(object):
    ALGORITHMS = {}

    @classmethod
    def register(cls, algoname, algo):
        cls.ALGORITHMS[algoname] = algo

    @classmethod
    def get_suppported_algorithms(cls):
        return cls.ALGORITHMS.keys()

    @classmethod
    def get(cls, algoname):
        return cls.ALGORITHMS[algoname]


SupportedAlgorithms.register("arcsort", ArcSortAlgorithm)
SupportedAlgorithms.register("connect", ConnectAlgorithm)
SupportedAlgorithms.register("invert", InvertAlgorithm)
SupportedAlgorithms.register("map", MapAlgorithm)
# SupportedAlgorithms.register("minimize", MinimizeAlgorithm)
SupportedAlgorithms.register("project", ProjectAlgorithm)
# SupportedAlgorithms.register("push", PushAlgorithm)
SupportedAlgorithms.register("reverse", ReverseAlgorithm)
# SupportedAlgorithms.register("rmfinalepsilon", RmFinalEpsilonAlgorithm)
SupportedAlgorithms.register("shortestpath", ShortestPathAlgorithm)
# SupportedAlgorithms.register("compose", ComposeAlgorithm)

