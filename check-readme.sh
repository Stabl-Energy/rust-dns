#!/usr/bin/env bash
top_level_dir=$(
  cd "$(dirname $0)"
  pwd
)
set -e
set -x
"$top_level_dir/update-readme.sh" --filename Readme.md.tmp
diff Readme.md Readme.md.tmp || (
  echo "ERROR: Readme.md is stale" >&2
  exit 1
)
rm -f Readme.md.tmp
git rm -f --ignore-unmatch Readme.md.tmp
