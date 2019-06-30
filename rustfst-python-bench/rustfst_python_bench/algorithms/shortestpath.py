class ShortestPathAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstshortestpath"

    @classmethod
    def rustfst_subcommand(cls):
        return "shortestpath"

