#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "=============================================="
echo "  IRIS Benchmarks Game Suite"
echo "  $(date '+%Y-%m-%d %H:%M:%S')"
echo "=============================================="
echo ""

# Build once in release mode



echo ""

# Run each benchmark
for bench in n-body spectral-norm fannkuch-redux binary-trees fasta reverse-complement k-nucleotide pidigits regex-redux thread-ring; do
    echo "----------------------------------------------"
    if [ -f "benchmark/$bench/run.sh" ]; then
        bash "benchmark/$bench/run.sh"
    else
        echo "  SKIP: benchmark/$bench/run.sh not found"
    fi
    echo ""
done

echo "=============================================="
echo "  Suite Complete"
echo "=============================================="
