#!/bin/bash

# This is a standalone script so we terminate as soon as anything
# errors. See https://github.com/travis-ci/travis-ci/issues/1066
set -e
set -x

export PATH=$PATH:~/.cargo/bin

echo "Checking formatting"
cd "rust_src"
cargo fmt -- --version

cargo fmt --all -- --check
