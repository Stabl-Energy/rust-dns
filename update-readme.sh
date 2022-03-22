#!/usr/bin/env bash
set -e
usage() {
  echo "$(basename "$0"): ERROR: $1" >&2
  echo usage: "$(basename "$0")" '[--directory DIRECTORY] [--filename FILENAME]' >&2
  exit 1
}

filename=Readme.md
while [ $# -gt 0 ]; do
  case "$1" in
  --filename)
    shift
    [ -n "$1" ] || usage "missing parameter to --filename argument"
    filename="$1"
    ;;
  *) usage "bad argument '$1'" ;;
  esac
  shift
done

echo "PWD=$(pwd)"
set -x
cargo readme >"$filename"
set +x

if grep --quiet '//! ## Cargo Geiger Safety Report' src/lib.rs; then
  time (
    set -x
    cargo geiger --update-readme --readme-path "$filename" --output-format GitHubMarkdown
    set +x
    echo -n "cargo geiger done."
  )
fi
