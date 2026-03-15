#!/usr/bin/env bash
set -euo pipefail

if [[ ! -x "./bin/aoxc" ]]; then
  echo "Missing ./bin/aoxc. Run: make package-bin"
  exit 1
fi

./bin/aoxc node-bootstrap
./bin/aoxc produce-once --tx "local-smoke"
