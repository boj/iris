#!/usr/bin/env bash
# Bootstrap IRIS from the stage0 binary + precompiled JSON artifacts.
#
# The stage0 binary is the frozen Rust evaluator. It loads JSON-compiled
# IRIS programs and evaluates them. The JSON artifacts contain the
# tokenizer, parser, lowerer, and interpreter — all written in IRIS.
#
# Usage:
#   ./bootstrap/bootstrap.sh run <file.iris> [args...]
#   ./bootstrap/bootstrap.sh compile <file.iris> [-o output.json]
#   ./bootstrap/bootstrap.sh test [project_root]
#   ./bootstrap/bootstrap.sh direct <program.json> [args...]
#   ./bootstrap/bootstrap.sh rebuild [project_root]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE0="$SCRIPT_DIR/iris-stage0"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ ! -x "$STAGE0" ]; then
    echo "error: stage0 binary not found at $STAGE0" >&2
    echo "Run: cargo build --release --features syntax --bin iris-stage0" >&2
    echo "Then: cp target/release/iris-stage0 bootstrap/" >&2
    exit 1
fi

# Pass through to iris-stage0
exec "$STAGE0" "$@"
