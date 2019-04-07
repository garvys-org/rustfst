#!/bin/sh

set -ex

cargo build --all
cargo test --all
cargo check --benches --all # running benches on travis is useless
cargo doc --all