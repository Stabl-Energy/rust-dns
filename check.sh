#!/usr/bin/env bash
package="$(basename "$PWD")"
set -e
set -x
time cargo check --all-targets --all-features "--package=$package"
time cargo build --all-targets --all-features "--package=$package"
time cargo fmt "--package=$package" -- --check
time cargo clippy --all-targets --all-features "--package=$package" -- -D clippy::pedantic
time cargo test --all-targets --all-features "--package=$package"
time cargo test --doc "--package=$package"
../check-readme.sh
time cargo publish --dry-run "--package=$package" "$@"
echo "$0 finished"
