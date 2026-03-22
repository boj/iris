#!/usr/bin/env python3
"""Fannkuch-Redux (Benchmarks Game) - Python reference implementation."""

import sys
import time
from itertools import permutations

def count_flips(perm):
    """Count pancake flips until first element is 1."""
    perm = list(perm)
    flips = 0
    while perm[0] != 1:
        k = perm[0]
        perm[:k] = perm[:k][::-1]
        flips += 1
    return flips

def fannkuch(n):
    """Compute max flips and checksum over all permutations of [1..n]."""
    max_flips = 0
    checksum = 0
    for i, perm in enumerate(permutations(range(1, n + 1))):
        flips = count_flips(perm)
        max_flips = max(max_flips, flips)
        checksum += flips if i % 2 == 0 else -flips
    return max_flips, checksum

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 7
    start = time.perf_counter()
    max_flips, checksum = fannkuch(n)
    elapsed = time.perf_counter() - start
    print(f"N={n}")
    print(f"Max flips: {max_flips}")
    print(f"Checksum:  {checksum}")
    print(f"Time: {elapsed*1000:.1f}ms")
