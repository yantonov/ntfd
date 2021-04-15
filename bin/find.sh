#!/usr/bin/env sh
set -o errexit -o nounset

cd "$(dirname "$0")"

cd ..

ps -ef | grep '[n]tfd' | grep 'server' | awk '{print $2}'
