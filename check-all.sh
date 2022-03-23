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
cd "$(dirname $0)"
top_level_dir=$(pwd)
set -e
set -x

time cargo check --verbose
time cargo build --verbose
time cargo fmt --all -- --check
time cargo clippy --all-targets --all-features -- -D clippy::pedantic
time cargo test --verbose

for project in $projects ; do
  cd "$top_level_dir/$project/"
  ../check-readme.sh
done

for project in $projects; do
  (cat "$top_level_dir/$project/Cargo.toml" |grep 'publish = false' >/dev/null) && continue || true;
  cd "$top_level_dir/$project/"
  time cargo publish --dry-run "$@"
done

echo "$0 finished"
