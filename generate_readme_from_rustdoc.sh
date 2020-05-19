#!/bin/sh

set -e
cd rustfst
cargo install cargo-sync-readme
cargo sync-readme -f lib
