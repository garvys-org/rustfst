from pathlib import Path
import shutil
import yaml

FILES_FORBIDDEN_LIST = ["__init__.py", "ffi_utils.py"]
DIR_FORBIDDEN_LIST = []


# Returns True if it contains python files
def rec(parent_path: Path, parent_package: str, output_path: Path) -> bool:
    contains_py_files = False

    for p in parent_path.iterdir():
        if (
            p.is_file()
            and str(p).endswith(".py")
            and p.name not in FILES_FORBIDDEN_LIST
        ):
            dir_path = output_path / p.stem
            dir_path.mkdir(parents=True)
            (dir_path / "index.md").write_text(f"::: {parent_package}.{p.stem}")
            contains_py_files = True
        elif p.is_dir() and p.name not in DIR_FORBIDDEN_LIST:
            contains_py_files_curr = rec(
                p, f"{parent_package}.{p.stem}", output_path / p.stem
            )
            if contains_py_files_curr:
                (output_path / p.stem / "index.md").write_text(
                    f"::: {parent_package}.{p.stem}"
                )
            contains_py_files |= contains_py_files_curr

    return contains_py_files


def generate_yaml(docs_path: Path, parent_path: Path):
    l_files = []
    l_dir = []
    for p in parent_path.iterdir():
        if p.is_file():
            l_files.append(str(p.relative_to(docs_path)))
        if p.is_dir():
            l_dir.append(generate_yaml(docs_path, p))
    return {f"{parent_path.stem}": l_files + l_dir}


def main():
    path_root = Path(__file__).resolve().parents[1].resolve()
    path_py_package = path_root / "rustfst-python" / "rustfst"
    path_mkdocs_yml = path_root / "rustfst-python" / "mkdocs.yml"

    output_root = path_root / "rustfst-python" / "docs" / "rustfst"
    if output_root.exists():
        shutil.rmtree(output_root)
    output_root.mkdir()

    # Generate md files
    rec(path_py_package, "rustfst", output_root)

    # Reference md files in the mkdocs.yml
    data = [
        {"Home": "index.md"},
        {
            "Code Reference": [
                generate_yaml(path_root / "rustfst-python" / "docs", output_root)
            ]
        },
    ]

    with path_mkdocs_yml.open() as f:
        mkdocs_yaml = yaml.full_load(f)

    mkdocs_yaml["nav"] = data

    with path_mkdocs_yml.open(mode="w") as f:
        yaml.dump(mkdocs_yaml, f)


if __name__ == "__main__":
    main()
