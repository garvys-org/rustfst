class ReverseAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstreverse"

    @classmethod
    def rustfst_subcommand(cls):
        return "reverse"

    @classmethod
    def get_parameters(cls):
        return [""]
