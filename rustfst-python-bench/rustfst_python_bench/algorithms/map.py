class MapAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstmap"

    @classmethod
    def rustfst_subcommand(cls):
        return "map"

    @classmethod
    def get_parameters(cls):
        map_types = ["arc_sum", "arc_unique", "identity", "input_epsilon", "invert", "output_epsilon", "rmweight"]
        return ["--map_type=" + m for m in map_types]
