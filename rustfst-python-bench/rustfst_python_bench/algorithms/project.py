from rustfst_python_bench.utils import check_fst_equals


class ProjectAlgorithm:
    def __init__(self, project_output=False):
        self.project_output = project_output

    @classmethod
    def openfst_cli(cls):
        return "fstproject"

    @classmethod
    def rustfst_subcommand(cls):
        return "project"

    def get_openfst_bench_cli(self):
        if self.project_output:
            return "bench_project", ["1"]
        else:
            return "bench_project", ["0"]

    def get_cli_args(self):
        if self.project_output:
            return "--project_output=true"
        else:
            return ""

    @classmethod
    def get_parameters(cls):
        return [cls(project_output=False), cls(project_output=True)]

    def check_correctness(self, path_res_openfst, path_res_rustfst):
        check_fst_equals(path_res_openfst, path_res_rustfst)
