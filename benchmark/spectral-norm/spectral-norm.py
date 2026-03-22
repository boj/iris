#!/usr/bin/env python3
"""Spectral Norm (Benchmarks Game) - Python reference implementation."""

import sys
import math
import time

def mat_a(i, j):
    return 1.0 / ((i + j) * (i + j + 1) // 2 + i + 1)

def mat_vec_a(u, n):
    return [sum(mat_a(i, j) * u[j] for j in range(n)) for i in range(n)]

def mat_vec_at(u, n):
    return [sum(mat_a(j, i) * u[j] for j in range(n)) for i in range(n)]

def at_a_u(u, n):
    return mat_vec_at(mat_vec_a(u, n), n)

def spectral_norm(n):
    u = [1.0] * n
    for _ in range(10):
        v = at_a_u(u, n)
        u = at_a_u(v, n)
    vBv = sum(ui * vi for ui, vi in zip(u, v))
    vv = sum(vi * vi for vi in v)
    return math.sqrt(vBv / vv)

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    start = time.perf_counter()
    result = spectral_norm(n)
    elapsed = time.perf_counter() - start
    print(f"N={n}")
    print(f"Result: {result:.9f}")
    print(f"Time: {elapsed*1000:.1f}ms")
