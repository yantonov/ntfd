#!/usr/bin/env sh
set -o errexit -o nounset

cd "$(dirname "$0")"

cd ..

cargo run server
