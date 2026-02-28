#!/bin/bash
set -e

# Check for argument
if [ -z "$1" ]; then
  echo "Usage: $0 <major|minor|patch>"
  exit 1
fi

# Check if cargo-edit is installed
if ! cargo --list | grep -q 'set-version'; then
  echo "cargo-edit not found, installing..."
  cargo install cargo-edit
fi

# Bump version
cargo set-version --bump "$1"

# Get new version
VERSION=$(grep '^version =' Cargo.toml | awk -F'"' '{print $2}')

# Commit and tag
git add Cargo.toml Cargo.lock
git commit -m "Bump version to $VERSION"
git tag "v$VERSION"
git push
git push --tags

echo "Version bumped to $VERSION and pushed."
