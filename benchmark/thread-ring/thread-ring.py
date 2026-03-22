#!/usr/bin/env python3
"""Thread Ring benchmark — pass a token around N threads."""

import sys
import time


def thread_ring(n_threads: int, token: int) -> int:
    """Simulate a thread ring: return the 1-indexed ID of the thread
    that sees the token reach 0."""
    for step in range(token + 1):
        if token == 0:
            return step % n_threads + 1
        token -= 1
    return 0  # shouldn't reach here


def main():
    n_threads = 503
    token = int(sys.argv[1]) if len(sys.argv) > 1 else 1000

    start = time.time()
    winner = thread_ring(n_threads, token)
    elapsed = time.time() - start

    print(f"Thread Ring (N={n_threads}, token={token})")
    print(f"  Winner: thread {winner}")
    print(f"  Time: {elapsed*1000:.1f}ms")


if __name__ == '__main__':
    main()
