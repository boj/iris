#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-100}"
echo "=== Spectral Norm Benchmark (N=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time cargo run --release --quiet -- run benchmark/spectral-norm/spectral-norm.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/spectral-norm/spectral-norm.py "$N"
else
    echo "(python3 not available)"
fi
