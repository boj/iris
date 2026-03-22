#!/usr/bin/env python3
"""Binary Trees (Benchmarks Game) - Python reference implementation.

Flat array representation to match the IRIS implementation.
"""

import sys
import time

def make_flat_tree(depth, item):
    """Build a flat complete binary tree as a list of item values."""
    size = 2 ** (depth + 1) - 1
    return [item + i for i in range(size)]

def flat_tree_check(tree, depth):
    """Compute checksum bottom-up. Leaf = value, Branch = value + left - right."""
    size = len(tree)
    checks = list(tree)  # Start with node values (correct for leaves)
    # Process levels from deepest internal to root
    for level in range(depth - 1, -1, -1):
        level_start = 2 ** level - 1
        level_end = 2 ** (level + 1) - 1
        for i in range(level_start, level_end):
            left_idx = 2 * i + 1
            right_idx = 2 * i + 2
            if left_idx < size:
                checks[i] = tree[i] + checks[left_idx] - checks[right_idx]
    return checks[0]

def bench_sum(depth):
    """Sum all node values in a flat tree."""
    tree = make_flat_tree(depth, 1)
    return sum(tree)

if __name__ == "__main__":
    depth = int(sys.argv[1]) if len(sys.argv) > 1 else 10
    start = time.perf_counter()

    tree = make_flat_tree(depth, 1)
    check = flat_tree_check(tree, depth)
    total = sum(tree)

    elapsed = time.perf_counter() - start
    print(f"Depth={depth}")
    print(f"Root checksum: {check}")
    print(f"Sum of nodes:  {total}")
    print(f"Time: {elapsed*1000:.1f}ms")
