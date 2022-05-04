import os
from pathlib import Path

import pytest
from pylint.lint import Run

ROOT_PATH = Path(__file__).parents[1]
RCFILEPATH = ROOT_PATH / "linting" / "pylintrc"


def run_linting_test(package):
    args = ["--rcfile", str(RCFILEPATH)]
    args += all_python_files(package)

    run = Run(args, exit=False)
    assert run.linter.msg_status == 0


def all_python_files(package_path):
    files = []
    for dirpath, _, filenames in os.walk(str(package_path)):
        files += [os.sep.join([dirpath, f]) for f in filenames if f.endswith(".py")]
    return files


def test_code():
    run_linting_test(ROOT_PATH / "rustfst")


def test_tests():
    run_linting_test(ROOT_PATH / "tests")
