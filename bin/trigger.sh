#!/usr/bin/env sh
set -o errexit -o nounset

curl -X POST 'http://localhost:4242/notify/ping'
