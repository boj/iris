#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-10}"
echo "=== Binary Trees Benchmark (depth=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time cargo run --release --quiet -- run benchmark/binary-trees/binary-trees.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/binary-trees/binary-trees.py "$N"
else
    echo "(python3 not available)"
fi
