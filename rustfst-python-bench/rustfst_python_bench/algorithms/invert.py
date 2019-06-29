class InvertAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstinvert"

    @classmethod
    def rustfst_subcommand(cls):
        return "invert"

