#!/usr/bin/env bash
# Regenerate the bootstrap JSON artifacts from IRIS source.
#
# Uses the stage0 binary to compile the self-hosted syntax pipeline
# and interpreter. The resulting JSON files are what stage0 loads
# at runtime to process .iris source files.
#
# This script is the self-hosting test: if the outputs are identical
# to the committed JSON files, the IRIS-written compiler is a stable
# fixed point.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE0="$SCRIPT_DIR/iris-stage0"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PROGRAMS="$PROJECT_ROOT/src/iris-programs"

if [ ! -x "$STAGE0" ]; then
    echo "error: stage0 binary not found" >&2
    exit 1
fi

echo "=== Rebuilding bootstrap JSON from IRIS source ==="

echo "  tokenizer.iris -> tokenizer.json"
"$STAGE0" compile "$PROGRAMS/syntax/tokenizer.iris" -o "$SCRIPT_DIR/tokenizer.json"

echo "  iris_parser.iris -> parser.json"
"$STAGE0" compile "$PROGRAMS/syntax/iris_parser.iris" -o "$SCRIPT_DIR/parser.json"

echo "  iris_lowerer.iris -> lowerer.json"
"$STAGE0" compile "$PROGRAMS/syntax/iris_lowerer.iris" -o "$SCRIPT_DIR/lowerer.json"

echo "  full_interpreter.iris -> interpreter.json"
"$STAGE0" compile "$PROGRAMS/interpreter/full_interpreter.iris" -o "$SCRIPT_DIR/interpreter.json"

echo "=== Done ==="
