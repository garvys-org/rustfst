import os
import sys
from pathlib import Path

from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension

packages = [p for p in find_packages() if "tests" not in p]

root = Path(__file__).resolve().parent.parent

readme_path = root / "README.md"
readme = None
if readme_path.exists():
    with readme_path.open() as f:
        readme = f.read()

PACKAGE_NAME = "rustfst-python"
RUST_EXTENSION_NAME = "rustfst.dylib.dylib"
VERSION = "1.1.2"
REPO_ROOT_PATH = Path(__file__).resolve().parents[1]
CARGO_ROOT_PATH = REPO_ROOT_PATH / "rustfst-ffi"
CARGO_FILE_PATH = CARGO_ROOT_PATH / "Cargo.toml"
CARGO_TARGET_DIR = REPO_ROOT_PATH / "target"
os.environ["CARGO_TARGET_DIR"] = str(CARGO_TARGET_DIR)

if "PROFILE" in os.environ:
    if os.environ.get("PROFILE") == "release":
        is_debug_profile = False
    elif os.environ.get("PROFILE") == "debug":
        is_debug_profile = True
    else:
        print("Invalid PROFILE %s" % os.environ.get("PROFILE"))
        sys.exit(1)
else:
    is_debug_profile = "develop" in sys.argv

setup(
    name=PACKAGE_NAME,
    version=VERSION,
    description="Library for constructing, combining, optimizing, and searching weighted finite-state "
    "transducers (FSTs). Re-implementation of OpenFst in Rust.",
    long_description=readme,
    long_description_content_type="text/markdown",
    extras_require={"tests": ["pytest>=6,<7"]},
    options={"bdist_wheel": {"universal": True}},
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Rust",
        "Topic :: Scientific/Engineering :: Mathematics",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Text Processing",
        "License :: OSI Approved :: Apache Software License",
    ],
    packages=packages,
    package_data={"rustfst": ["py.typed"]},
    include_package_data=True,
    rust_extensions=[
        RustExtension(
            RUST_EXTENSION_NAME,
            str(CARGO_FILE_PATH),
            debug=is_debug_profile,
            binding=Binding.NoBinding,
        )
    ],
    zip_safe=False,
    url="https://github.com/garvys-org/rustfst",
    author="Alexandre Caulier, Emrick Sinitambirivoutin",
    author_email="alexandre.caulier.a@gmail.com, emrick.sinitambirivoutin@sonos.com",
    keywords="fst openfst graph transducer acceptor shortest-path minimize determinize wfst",
    project_urls={
        "Documentation": "https://garvys-org.github.io/rustfst/",
        "Source": "https://github.com/garvys-org/rustfst",
    },
    python_requires=">=3.7",
    license="Apache License, Version 2.0",
)
