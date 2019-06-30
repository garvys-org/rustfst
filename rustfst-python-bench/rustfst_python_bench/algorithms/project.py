class ProjectAlgorithm:
    def __init__(self):
        pass

    @classmethod
    def openfst_cli(cls):
        return "fstproject"

    @classmethod
    def rustfst_subcommand(cls):
        return "project"

    @classmethod
    def get_parameters(cls):
        return ["", "--project_output"]
