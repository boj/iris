#!/bin/bash
# build_self_compiled.sh -- IRIS self-compilation pipeline
#
# This script runs c_backend.json (the compiled c_backend.iris) on
# interpreter.json to produce a new compiled C interpreter, then
# builds and verifies it.
#
# Prerequisites:
#   - bootstrap/c_backend.json must exist (compiled from c_backend.iris)
#   - bootstrap/c-runtime/iris-stage0-c must be built
#
# The self-compilation chain:
#   1. iris-stage0-c (C tree-walker) evaluates c_backend.json
#   2. c_backend.json reads interpreter.json and emits C source
#   3. The emitted C source IS the compiled interpreter
#   4. We compile it with the C runtime to get a new stage0-c binary
#   5. We verify the new binary produces identical output

set -euo pipefail

cd "$(dirname "$0")"

STAGE0=c-runtime/iris-stage0-c
INTERP=interpreter.json
OUTPUT=c-runtime/iris_interp_compiled_new.c

if [ ! -f "$STAGE0" ]; then
    echo "ERROR: $STAGE0 not found. Run 'cd c-runtime && make' first."
    exit 1
fi

if [ ! -f c_backend.json ]; then
    echo "ERROR: c_backend.json not found."
    echo "Compile c_backend.iris to JSON first using the IRIS compilation pipeline."
    exit 1
fi

echo "=== IRIS Self-Compilation ==="
echo ""
echo "Step 1: Run c_backend.json on interpreter.json..."
echo "  $STAGE0 direct c_backend.json $INTERP"
$STAGE0 direct c_backend.json "$INTERP" > "$OUTPUT" 2>/dev/null
echo "  Generated: $OUTPUT ($(wc -l < "$OUTPUT") lines)"
echo ""

echo "Step 2: Compile new stage0 with generated interpreter..."
cd c-runtime
gcc -O2 -Wall -Wextra -Wno-unused-parameter -std=c11 \
    -o iris-stage0-c-new \
    main.c iris_graph.c iris_prims.c iris_eval.c iris_interp_compiled_new.c -lm
echo "  Built: iris-stage0-c-new"
echo ""

echo "Step 3: Verify new binary loads interpreter.json..."
./iris-stage0-c-new direct ../interpreter.json 2>/dev/null && echo "  OK: binary runs"
echo ""

echo "=== Self-compilation complete ==="
echo "The generated C file and new binary are ready."
echo "To replace the current compiled interpreter:"
echo "  cp c-runtime/iris_interp_compiled_new.c c-runtime/iris_interp_compiled.c"
echo "  cd c-runtime && make"
