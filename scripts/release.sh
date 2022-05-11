#!/usr/bin/env bash
set -e

NEW_VERSION=$1

NEW_TAG="rustfst-v$NEW_VERSION"

./update_version.sh $NEW_VERSION
git commit -am "Tag $NEW_VERSION"
git push
git tag -a $NEW_TAG -m "Release rustfst $NEW_VERSION"
git push --tags
