#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."

N="${1:-1000}"
echo "=== FASTA Benchmark (N=$N) ==="

echo ""
echo "--- IRIS (interpreter) ---"
time cargo run --release --quiet -- run benchmark/fasta/fasta.iris "$N" 2>&1 || echo "(IRIS run failed)"

echo ""
echo "--- Python 3 ---"
if command -v python3 &>/dev/null; then
    python3 benchmark/fasta/fasta.py "$N"
else
    echo "(python3 not available)"
fi
