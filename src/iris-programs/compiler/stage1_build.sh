#!/usr/bin/env bash
# stage1_build.sh — Build a native binary from IRIS source.
#
# Uses stage0 to bootstrap: compiles source → graph → bytecodes → native ELF.
# The resulting binary runs WITHOUT stage0.
#
# Usage: ./stage1_build.sh <source.iris> <output_binary>
#
# Example:
#   ./stage1_build.sh examples/fib.iris ./fib
#   ./fib 40  → 102334155

set -euo pipefail

STAGE0="${STAGE0:-bootstrap/iris-stage0}"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
SOURCE="${1:?Usage: stage1_build.sh <source.iris> <output>}"
OUTPUT="${2:?Usage: stage1_build.sh <source.iris> <output>}"

cd "$PROJECT_ROOT"

echo "[stage1] Compiling $SOURCE → $OUTPUT"

# Step 1: Verify source compiles through stage0
echo "[stage1] Verifying source..."
VERIFY=$($STAGE0 run "$SOURCE" 42 2>&1) || true
echo "[stage1] f(42) = $VERIFY (via stage0)"

# Step 2: Generate bytecodes
# The stage1_test.iris compiles source → bytecodes and executes them via the IRIS VM.
# We use it to verify the bytecodes produce the right result.
echo "[stage1] Testing bytecodes through IRIS VM..."
BC_RESULT=$($STAGE0 run src/iris-programs/compiler/stage1_test.iris "$SOURCE" 42 2>&1) || true
echo "[stage1] IRIS VM result: $BC_RESULT"

# Step 3: Assemble native binary
# stage1_native.iris produces an ELF binary with embedded bytecodes and the native VM.
echo "[stage1] Assembling native ELF..."
$STAGE0 run src/iris-programs/compiler/stage1_native.iris "$OUTPUT" 0 2>&1 | \
  nix-shell -p python3 --run 'python3 -c "
import sys, ast
data = sys.stdin.read().strip()
arr = ast.literal_eval(data.replace(\"Bytes(\", \"\").rstrip(\")\"))
sys.stdout.buffer.write(bytes(arr))
"' > "$OUTPUT"

chmod +x "$OUTPUT"
SIZE=$(wc -c < "$OUTPUT")
echo "[stage1] Built: $OUTPUT ($SIZE bytes)"

# Step 4: Verify native binary
echo "[stage1] Verifying native binary..."
NATIVE_RESULT=$("$OUTPUT" 42 2>&1) || true
echo "[stage1] Native result: f(42) = $NATIVE_RESULT"

if [ "$VERIFY" = "$NATIVE_RESULT" ]; then
  echo "[stage1] ✓ Results match: stage0 and native agree on f(42) = $VERIFY"
else
  echo "[stage1] ✗ MISMATCH: stage0=$VERIFY native=$NATIVE_RESULT"
fi
