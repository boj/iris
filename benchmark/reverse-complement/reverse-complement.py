#!/usr/bin/env python3
"""Reverse Complement (Benchmarks Game) - Python reference implementation."""

import sys
import time

COMPLEMENT = str.maketrans("ACGTacgt", "TGCAtgca")

def reverse_complement(dna):
    return dna[::-1].translate(COMPLEMENT)

def reverse_complement_slow(dna):
    """Character-by-character version matching the IRIS implementation."""
    comp = {"A": "T", "T": "A", "C": "G", "G": "C",
            "a": "t", "t": "a", "c": "g", "g": "c"}
    result = []
    for i in range(len(dna) - 1, -1, -1):
        result.append(comp.get(dna[i], dna[i]))
    return "".join(result)

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
    # Generate a test DNA string
    bases = "ACGT"
    dna = "".join(bases[i % 4] for i in range(n))

    start = time.perf_counter()
    rc = reverse_complement_slow(dna)
    elapsed = time.perf_counter() - start

    print(f"N={n}")
    print(f"Input prefix:  {dna[:60]}")
    print(f"Output prefix: {rc[:60]}")
    print(f"Length: {len(rc)}")
    # Verify roundtrip
    assert reverse_complement_slow(rc) == dna, "roundtrip failed"
    print("Roundtrip: OK")
    print(f"Time: {elapsed*1000:.1f}ms")
