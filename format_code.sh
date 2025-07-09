#!/usr/bin/env bash
set -e

cargo fmt
uv --directory rustfst-python tool run ruff format . && echo "Code formatted !"