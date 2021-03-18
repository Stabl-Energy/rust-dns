#!/usr/bin/env bash
set -e
set -x
"$(dirname "$0")"/check-all.sh +stable "$@"
git push --follow-tags
