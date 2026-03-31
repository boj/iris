#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-7}"
echo "=== Fannkuch-Redux Benchmark (N=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time bootstrap/iris-stage0 run benchmark/fannkuch-redux/fannkuch-redux.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/fannkuch-redux/fannkuch-redux.py "$N"
else
    echo "(python3 not available)"
fi
