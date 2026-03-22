#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

TOKEN="${1:-1000}"
echo "=== Thread Ring Benchmark (token=$TOKEN) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time cargo run --release --quiet -- run benchmark/thread-ring/thread-ring.iris "$TOKEN" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/thread-ring/thread-ring.py "$TOKEN"
else
    echo "(python3 not available)"
fi
