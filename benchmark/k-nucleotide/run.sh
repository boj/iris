#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-100}"
echo "=== K-Nucleotide Benchmark (N=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time bootstrap/iris-stage0 run benchmark/k-nucleotide/k-nucleotide.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/k-nucleotide/k-nucleotide.py "$N"
else
    echo "(python3 not available)"
fi
