class ArcSortAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstarcsort"

    @classmethod
    def rustfst_subcommand(cls):
        return "arcsort"

    @classmethod
    def get_parameters(cls):
        return ["--sort_type=ilabel", "--sort_type=olabel"]
