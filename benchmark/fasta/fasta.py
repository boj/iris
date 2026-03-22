#!/usr/bin/env python3
"""FASTA (Benchmarks Game) - Python reference implementation.

Simplified version matching the IRIS implementation.
"""

import sys
import time

IM = 139968
IA = 3877
IC = 29573

ALU = "GGCCGGGCGCGGTGGCTCACGCCTGTAATCCCAGCACTTTGGGAGGCCGAGGCGGGCGGATCACCTGAGGTCAGGAGTTCGAG"

def lcg_next(seed):
    new_seed = (seed * IA + IC) % IM
    return new_seed, new_seed / IM

def select_nucleotide(r):
    if r < 0.27:
        return "A"
    elif r < 0.39:
        return "C"
    elif r < 0.51:
        return "G"
    else:
        return "T"

def random_dna(n, seed):
    result = []
    for _ in range(n):
        seed, r = lcg_next(seed)
        result.append(select_nucleotide(r))
    return seed, "".join(result)

def repeat_string(base, n):
    result = []
    for i in range(n):
        result.append(base[i % len(base)])
    return "".join(result)

def fasta(n):
    alu_out = repeat_string(ALU, n)
    final_seed, dna_out = random_dna(n, 42)
    return alu_out, dna_out, final_seed

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
    start = time.perf_counter()
    alu_out, dna_out, final_seed = fasta(n)
    elapsed = time.perf_counter() - start
    print(f"N={n}")
    print(f"ALU length: {len(alu_out)}")
    print(f"DNA length: {len(dna_out)}")
    print(f"Final seed: {final_seed}")
    print(f"DNA prefix: {dna_out[:60]}")
    print(f"Time: {elapsed*1000:.1f}ms")
