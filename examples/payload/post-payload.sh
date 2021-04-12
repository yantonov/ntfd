#!/usr/bin/env sh
set -o errexit -o nounset

curl -X POST --data '{"foo": "bar", "bar": "baz"}' 'http://localhost:4242/notify/ping'
