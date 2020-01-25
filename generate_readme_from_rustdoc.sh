#!/bin/sh

set -e
cd rustfst
cargo sync-readme -f lib
