#!/usr/bin/env bash
# Use bash because because it has a built-in 'time' command.
set -e
cd "$(dirname "$0")"
echo "PWD=$(pwd)"
time (
  set -x
  ./check.sh any-range
  ./check.sh build-data "$@"
  ./check.sh build-data-test
  ./check.sh fair-rate-limiter
  ./check.sh permit
  ./check.sh prob-rate-limiter
  ./check.sh rustls-pin
  ./check.sh safe-dns
  ./check.sh safe-lock
  ./check.sh temp-dir
  ./check.sh temp-file
  set +x
  echo -n "$(basename "$0") finished."
)
