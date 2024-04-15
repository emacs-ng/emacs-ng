#!/bin/bash

if [[ -z $@ ]]; then
    echo "Usage: ./vendor_cargo_source.sh <emacs_version>"
fi

mkdir -p .cargo
cargo vendor ./third_party/rust --respect-source-config > \
      .cargo/config.toml

git config --global user.email "bot@github.io"
git config --global user.name "github bot"
git config tar.tar.xz.command "xz -c"
git add -f ./third_party/rust
git add .cargo/config.toml
git commit --no-verify -m "Vendor Cargo Source"

prefix=vendored-source_$1

git archive --prefix=$prefix/ HEAD -o $prefix.tar.xz
