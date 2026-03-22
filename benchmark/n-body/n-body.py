#!/usr/bin/env python3
"""N-Body simulation (Benchmarks Game) - Python reference implementation.

Simplified 2-body version (Sun + Jupiter) to match the IRIS implementation.
"""

import sys
import math
import time

SOLAR_MASS = 4 * math.pi ** 2
DAYS_PER_YEAR = 365.24
DT = 0.01

# Sun at origin
SUN = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, SOLAR_MASS]

# Jupiter
JUPITER = [
    4.84143144246472090e+00,
    -1.16032004402742839e+00,
    -1.03622044471123109e-01,
    1.66007664274403694e-03 * DAYS_PER_YEAR,
    7.69901118419740425e-03 * DAYS_PER_YEAR,
    -6.90107685045669070e-05 * DAYS_PER_YEAR,
    9.54791938424326609e-04 * SOLAR_MASS,
]

def kinetic_energy(vx, vy, vz, mass):
    return 0.5 * mass * (vx*vx + vy*vy + vz*vz)

def potential_energy(x1, y1, z1, m1, x2, y2, z2, m2):
    dx = x1 - x2
    dy = y1 - y2
    dz = z1 - z2
    dist = math.sqrt(dx*dx + dy*dy + dz*dz)
    return -(m1 * m2) / dist

def energy(sun, jup):
    ke = kinetic_energy(sun[3], sun[4], sun[5], sun[6])
    ke += kinetic_energy(jup[3], jup[4], jup[5], jup[6])
    pe = potential_energy(sun[0], sun[1], sun[2], sun[6],
                         jup[0], jup[1], jup[2], jup[6])
    return ke + pe

def advance(sun, jup, dt):
    dx = sun[0] - jup[0]
    dy = sun[1] - jup[1]
    dz = sun[2] - jup[2]
    d2 = dx*dx + dy*dy + dz*dz
    dist = math.sqrt(d2)
    mag = dt / (d2 * dist)

    sun[3] -= dx * jup[6] * mag
    sun[4] -= dy * jup[6] * mag
    sun[5] -= dz * jup[6] * mag

    jup[3] += dx * sun[6] * mag
    jup[4] += dy * sun[6] * mag
    jup[5] += dz * sun[6] * mag

    sun[0] += dt * sun[3]
    sun[1] += dt * sun[4]
    sun[2] += dt * sun[5]

    jup[0] += dt * jup[3]
    jup[1] += dt * jup[4]
    jup[2] += dt * jup[5]

def main(n):
    sun = list(SUN)
    jup = list(JUPITER)

    e0 = energy(sun, jup)

    for _ in range(n):
        advance(sun, jup, DT)

    e1 = energy(sun, jup)
    return e0, e1

if __name__ == "__main__":
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 1000
    start = time.perf_counter()
    e0, e1 = main(n)
    elapsed = time.perf_counter() - start
    print(f"N={n}")
    print(f"Initial energy: {e0:.9f}")
    print(f"Final energy:   {e1:.9f}")
    print(f"Time: {elapsed*1000:.1f}ms")
