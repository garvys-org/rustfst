#!/bin/sh

set -ex

cargo build
cargo test
cargo check --benches # running benches on travis is useless
cargo doc