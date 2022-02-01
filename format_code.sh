#!/usr/bin/env bash
set -e

# Format Rust code
cargo fmt && echo "Rust code formatted !"

# Format python code
black .

# Check python linting
python -m pytest --cache-clear --disable-warnings rustfst-python/linting/linting_test.py