#!/usr/bin/env bash

. "$(dirname "$0")"/lib.sh

set -e
ls -1 Readme.md
generate_readme Readme.md