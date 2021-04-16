#!/usr/bin/env bash
set -e
script_dir="$(
  cd "$(dirname "$0")"
  pwd
)"
set -x
"$script_dir"/check-all.sh
git push --follow-tags
