#!/bin/bash

set -e

NEW_VERSION=$1

if [ -z $NEW_VERSION ]; then
    echo "Usage: $0 NEW_VERSION"
    exit 1
fi

echo "NEW VERSION $NEW_VERSION" >&2

perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" rustfst/Cargo.toml
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" rustfst-ffi/Cargo.toml
perl -p -i -e "s/version = \".*\" }\$/version = \"=$NEW_VERSION\" }/g" rustfst-ffi/Cargo.toml
perl -p -i -e "s/^version = \".*\"\$/version = \"$NEW_VERSION\"/g" rustfst-cli/Cargo.toml
perl -p -i -e "s/^VERSION = \".*\"\$/VERSION = \"$NEW_VERSION\"/g" rustfst-python/setup.py