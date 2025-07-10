#!/usr/bin/env bash
set -e

cargo fmt
uv tool run ruff check . || { echo "Code not formatted! Please run 'uv tool run ruff format .'"; exit 1; }
uv --directory rustfst-python tool run ruff format . && echo "Code formatted !"