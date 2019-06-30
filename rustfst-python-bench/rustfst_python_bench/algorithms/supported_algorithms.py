
from rustfst_python_bench.algorithms.connect import ConnectAlgorithm
from rustfst_python_bench.algorithms.invert import InvertAlgorithm
from rustfst_python_bench.algorithms.project import ProjectAlgorithm
from rustfst_python_bench.algorithms.minimize import MinimizeAlgorithm
from rustfst_python_bench.algorithms.shortestpath import ShortestPathAlgorithm


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


SupportedAlgorithms.register("connect", ConnectAlgorithm)
SupportedAlgorithms.register("invert", InvertAlgorithm)
SupportedAlgorithms.register("project", ProjectAlgorithm)
SupportedAlgorithms.register("minimize", MinimizeAlgorithm)
SupportedAlgorithms.register("shortestpath", ShortestPathAlgorithm)
