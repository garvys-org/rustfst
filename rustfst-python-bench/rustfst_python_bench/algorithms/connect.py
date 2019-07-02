class ConnectAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstconnect"

    @classmethod
    def rustfst_subcommand(cls):
        return "connect"

    def get_openfst_bench_cli(self):
        return "bench_connect", []

    def get_cli_args(self):
        return ""

    @classmethod
    def get_parameters(cls):
        return [cls()]
