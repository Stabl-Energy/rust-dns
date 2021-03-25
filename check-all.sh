#!/usr/bin/env bash
# Use bash because because it has a built-in 'time' command.

. "$(dirname "$0")"/lib.sh

check_crate() {
  cd "$TOP_LEVEL_DIR/$1"
  if [ "$TOOLCHAIN_ARG" != '+nightly' ]; then
    cargo_fmt_clippy
    # Once cargo-geiger builds on nightly,
    # change this to always check the readme.
    # https://github.com/rust-secure-code/cargo-geiger/issues/181
    check_readme
  fi
  cargo_publish_dryrun
}

check_all() {
  cd "$TOP_LEVEL_DIR"
  cargo_check_build_test

  # https://github.com/rust-secure-code/cargo-geiger/issues/145
  SKIP_CARGO_GEIGER=1
  time check_crate rustls-pin
  SKIP_CARGO_GEIGER=

  time check_crate any-range
  time check_crate permit
  time check_crate safe-lock
  time check_crate temp-dir
  time check_crate temp-file
  echo "$0 finished"
}

set -e
set -x
time check_all "$@"
