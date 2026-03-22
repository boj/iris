#!/usr/bin/env python3
"""Regex-Redux benchmark — pattern matching and replacement on DNA."""

import sys
import time


def count_pattern(text: str, pattern: str) -> int:
    """Count non-overlapping occurrences of pattern in text."""
    count = 0
    start = 0
    while True:
        idx = text.find(pattern, start)
        if idx == -1:
            break
        count += 1
        start = idx + len(pattern)
    return count


# IUB code replacements
REPLACEMENTS = [
    ('B', 'CGT'), ('D', 'AGT'), ('H', 'ACT'),
    ('K', 'GT'), ('M', 'AC'), ('N', 'ACGT'),
    ('R', 'AG'), ('S', 'GC'), ('V', 'ACG'),
    ('W', 'AT'), ('Y', 'CT'),
]


def apply_replacements(dna: str) -> str:
    """Apply IUB code replacements."""
    for old, new in REPLACEMENTS:
        dna = dna.replace(old, new)
    return dna


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 100
    import random
    rng = random.Random(42)
    # Include some IUB codes
    alphabet = 'ACGTBDHKMNRSVWY'
    dna = ''.join(rng.choice(alphabet) for _ in range(n))

    start = time.time()

    original_len = len(dna)
    counts = {base: count_pattern(dna, base) for base in 'ACGT'}
    replaced = apply_replacements(dna)
    new_len = len(replaced)

    elapsed = time.time() - start
    print(f"Regex-Redux (N={n})")
    print(f"  Original length: {original_len}")
    print(f"  A={counts['A']}, T={counts['T']}, G={counts['G']}, C={counts['C']}")
    print(f"  After replacements: {new_len}")
    print(f"  Time: {elapsed*1000:.1f}ms")


if __name__ == '__main__':
    main()
