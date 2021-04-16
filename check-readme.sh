#!/usr/bin/env bash
set -e

script_dir="$(cd "$(dirname "$0")"; pwd)"

if [ $# -ne 1 ] || [ -z "$1" ]; then
  echo usage: "$(basename "$0")" 'DIRECTORY' >&2
  exit 1
fi

cd "$1"
"$script_dir/generate-readme.sh" --filename Readme.md.tmp

echo ''
echo "Checking if source readme matches generated one."
set -x
diff Readme.md Readme.md.tmp >&2 || exit 1
rm -f Readme.md.tmp
git rm -f --ignore-unmatch Readme.md.tmp
