class ShortestPathAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstshortestpath"

    @classmethod
    def rustfst_subcommand(cls):
        return "shortestpath"

    def get_openfst_bench_cli(self):
        raise NotImplementedError

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]

