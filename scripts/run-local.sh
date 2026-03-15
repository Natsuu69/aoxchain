#!/usr/bin/env bash
set -euo pipefail

cargo run -p aoxcmd -- node-bootstrap
cargo run -p aoxcmd -- produce-once --tx "local-smoke"
