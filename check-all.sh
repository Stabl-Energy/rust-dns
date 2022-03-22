#!/usr/bin/env bash
projects="any-range \
  build-data \
  build-data-test \
  fair-rate-limiter \
  permit \
  prob-rate-limiter \
  rustls-pin \
  safe-dns \
  safe-lock \
  temp-dir \
  temp-file"
top_level_dir=$(
  cd "$(dirname $0)"
  pwd
)
set -e
set -x

time cargo check --verbose
time cargo build --verbose
time cargo fmt --all -- --check
time cargo clippy --all-targets --all-features -- -D clippy::pedantic

for project in $projects ; do
  cd "$top_level_dir/$project/"
  "$top_level_dir/check-readme.sh"
done

cd "$top_level_dir/"
time cargo test --verbose

for project in $projects; do
  cd "$top_level_dir/$project/"
  time cargo publish --dry-run "$@"
done

echo "$0 finished"
