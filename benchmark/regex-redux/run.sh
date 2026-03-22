#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-100}"
echo "=== Regex-Redux Benchmark (N=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time cargo run --release --quiet -- run benchmark/regex-redux/regex-redux.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/regex-redux/regex-redux.py "$N"
else
    echo "(python3 not available)"
fi
