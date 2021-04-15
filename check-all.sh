#!/usr/bin/env bash
# Use bash because because it has a built-in 'time' command.

. "$(dirname "$0")"/lib.sh

non_nightly_checks() {
  if [ "$TOOLCHAIN_ARG" != '+nightly' ]; then
    cargo_fmt_clippy
    # Once cargo-geiger builds on nightly,
    # change this to always check the readme.
    # https://github.com/rust-secure-code/cargo-geiger/issues/181
    check_readme
  fi
}

check_crate() {
  cd "$TOP_LEVEL_DIR/$1"
  non_nightly_checks
  cargo_publish_dryrun
}

check_crate_sans_geiger() {
  cd "$TOP_LEVEL_DIR/$1"
  # https://github.com/rust-secure-code/cargo-geiger/issues/145
  SKIP_CARGO_GEIGER=1
  non_nightly_checks
  SKIP_CARGO_GEIGER=
  cargo_publish_dryrun
}

check_crate_sans_publish() {
  cd "$TOP_LEVEL_DIR/$1"
  non_nightly_checks
}

check_all() {
  cd "$TOP_LEVEL_DIR"
  cargo_check_build_test

  time check_crate any-range
  time check_crate build-data
  time check_crate_sans_publish build-data-test
  time check_crate permit
  time check_crate_sans_geiger rustls-pin
  time check_crate safe-lock
  time check_crate temp-dir
  time check_crate temp-file
  echo "$0 finished"
}

set -e
set -x
time check_all "$@"
