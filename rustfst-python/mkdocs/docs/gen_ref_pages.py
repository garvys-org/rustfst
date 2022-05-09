"""Generate the code reference pages."""

from pathlib import Path

import mkdocs_gen_files

path_rustfst_python = Path(__file__).resolve().parents[2].resolve()
path_rustfst_module = path_rustfst_python / "rustfst"
path_docs = path_rustfst_python / "mkdocs" / "docs"
path_reference = path_rustfst_python / "mkdocs" / "reference"

nav = mkdocs_gen_files.Nav()

for path in sorted(path_rustfst_module.rglob("*.py")):  #
    print(path)

    module_path = path.relative_to(path_rustfst_python).with_suffix("")  #
    print(f"Module path = {module_path}")

    doc_path = path.relative_to(path_rustfst_python).with_suffix(".md")  #
    print(f"Doc path = {doc_path}")

    full_doc_path = path_reference / doc_path  #

    parts = list(module_path.parts)
    print(f"Parts = {parts}")

    if parts[-1] == "__init__":  #

        parts = parts[:-1]

    elif parts[-1] == "__main__":
        continue

    nav[parts] = doc_path.as_posix()

    with mkdocs_gen_files.open(full_doc_path, "w") as fd:  #

        identifier = ".".join(parts)  #

        print("::: " + identifier, file=fd)  #

    print(f"Full doc path = {full_doc_path}")
    mkdocs_gen_files.set_edit_path(full_doc_path, path)

with mkdocs_gen_files.open(path_reference / "SUMMARY.md", "w") as nav_file:  #
    nav_file.writelines(nav.build_literate_nav())