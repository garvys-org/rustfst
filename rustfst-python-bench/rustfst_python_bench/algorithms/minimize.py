class MinimizeAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstminimize"

    @classmethod
    def rustfst_subcommand(cls):
        return "minimize"

