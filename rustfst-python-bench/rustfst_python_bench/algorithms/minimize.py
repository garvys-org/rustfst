class MinimizeAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstminimize"

    @classmethod
    def rustfst_subcommand(cls):
        return "minimize"

    @classmethod
    def get_parameters(cls):
        return ["--allow_nondet=true"]

