#!/usr/bin/env bash
# Bootstrap IRIS from the stage0 binary + precompiled JSON artifacts.
#
# The stage0 binary is the frozen Rust evaluator. It loads JSON-compiled
# IRIS programs and evaluates them. The JSON artifacts contain the
# tokenizer, parser, lowerer, and interpreter — all written in IRIS.
#
# The Lean kernel server (iris-kernel-server) must be built separately:
#   cd lean && lake build iris-kernel-server
#
# Usage:
#   ./bootstrap/bootstrap.sh run <file.iris> [args...]
#   ./bootstrap/bootstrap.sh test [dir]
#   ./bootstrap/bootstrap.sh direct <program.json> [args...]

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE0="$SCRIPT_DIR/iris-stage0"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

if [ ! -x "$STAGE0" ]; then
    echo "error: stage0 binary not found at $STAGE0" >&2
    echo "Run 'cargo build --release --features syntax' and copy to bootstrap/" >&2
    exit 1
fi

# Ensure the Lean kernel server is built
LEAN_SERVER="$PROJECT_ROOT/lean/.lake/build/bin/iris-kernel-server"
if [ ! -x "$LEAN_SERVER" ]; then
    echo "Building Lean kernel server..." >&2
    (cd "$PROJECT_ROOT/lean" && lake build iris-kernel-server)
fi

case "${1:-help}" in
    run)
        shift
        "$STAGE0" direct "$SCRIPT_DIR/tokenizer.json" "$@"
        ;;
    test)
        shift
        "$STAGE0" test "${1:-.}"
        ;;
    direct)
        shift
        "$STAGE0" direct "$@"
        ;;
    help|--help|-h)
        echo "Usage: bootstrap.sh <command> [args...]"
        echo ""
        echo "Commands:"
        echo "  run <file.iris> [args]  Compile and run an IRIS source file"
        echo "  test [dir]             Run the self-hosted test suite"
        echo "  direct <prog.json>     Evaluate a pre-compiled JSON program"
        ;;
    *)
        echo "Unknown command: $1" >&2
        echo "Run 'bootstrap.sh help' for usage" >&2
        exit 1
        ;;
esac
