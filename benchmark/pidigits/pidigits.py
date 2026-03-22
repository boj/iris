#!/usr/bin/env python3
"""Pi Digits benchmark — compute digits of pi using a spigot algorithm."""

import sys
import time


def pidigits(n: int) -> str:
    """Compute n digits of pi using the Gibbons spigot algorithm."""
    q, r, t, k, l = 1, 0, 1, 1, 3
    digits = []

    while len(digits) < n:
        # Extract digit
        nd = (3 * q + r) // t
        # Check safety
        if nd == (4 * q + r) // t:
            digits.append(str(nd))
            # Produce
            q, r = 10 * q, 10 * (r - nd * t)
        else:
            # Consume
            q, r, t = q * k, (2 * q + r) * l, t * l
            k += 1
            l += 2

    return ''.join(digits)


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 27
    start = time.time()
    result = pidigits(n)
    elapsed = time.time() - start
    print(f"Pi Digits (N={n})")
    print(f"  Result: {result}")
    print(f"  Time: {elapsed*1000:.1f}ms")


if __name__ == '__main__':
    main()
