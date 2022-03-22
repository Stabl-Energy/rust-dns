#!/usr/bin/env bash
package="$(basename "$PWD")"
set -e
set -x
time cargo check --verbose "--package=$package"
time cargo build --verbose "--package=$package"
time cargo fmt "--package=$package" -- --check
time cargo clippy --all-targets --all-features "--package=$package" -- -D clippy::pedantic
time cargo test --verbose "--package=$package"
../check-readme.sh
time cargo publish --dry-run "--package=$package" "$@"
echo "$0 finished"
