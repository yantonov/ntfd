#!/usr/bin/env sh
set -o errexit -o nounset

curl 'http://localhost:4242/notify/ping'
