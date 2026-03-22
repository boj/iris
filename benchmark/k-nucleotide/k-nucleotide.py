#!/usr/bin/env python3
"""K-Nucleotide benchmark — count DNA subsequence frequencies."""

import sys
from collections import Counter


def count_kmers(dna: str, k: int) -> dict[str, int]:
    """Count all k-mers of length k in a DNA string."""
    counts: dict[str, int] = {}
    for i in range(len(dna) - k + 1):
        kmer = dna[i:i+k]
        counts[kmer] = counts.get(kmer, 0) + 1
    return counts


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    # Generate a deterministic DNA string
    import random
    rng = random.Random(42)
    dna = ''.join(rng.choice('ACGT') for _ in range(n))

    import time
    start = time.time()

    freq1 = count_kmers(dna, 1)
    freq2 = count_kmers(dna, 2)

    elapsed = time.time() - start
    print(f"K-Nucleotide (N={n})")
    print(f"  1-mers: {len(freq1)} distinct")
    print(f"  2-mers: {len(freq2)} distinct")
    print(f"  GG count: {freq2.get('GG', 0)}")
    print(f"  Time: {elapsed*1000:.1f}ms")


if __name__ == '__main__':
    main()
