class ConnectAlgorithm:

    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstconnect"

    @classmethod
    def rustfst_subcommand(cls):
        return "connect"
