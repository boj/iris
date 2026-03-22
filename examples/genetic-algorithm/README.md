# Genetic Algorithm

A genetic algorithm written entirely in IRIS that evolves a sequence of numbers
to sum to a target value.

## How it works

This is NOT using iris-evolve. It is a complete GA implemented from scratch in
IRIS syntax using only built-in primitives:

- **Fitness**: absolute difference between sum of genes and target
- **Selection**: tournament selection (pick better of two random individuals)
- **Crossover**: single-point (first half from parent1, second from parent2)
- **Mutation**: replace a random gene with a random value

The population is a tuple of individuals, where each individual is a tuple
of integers. Evolution proceeds by evaluating fitness, selecting parents,
performing crossover, and mutating offspring.

## Running

```bash
iris run examples/genetic-algorithm/ga.iris
iris run examples/genetic-algorithm/ga-test.iris
```

## Primitives used

- `fold` / `map` / `filter` - functional iteration
- `list_take` / `list_drop` / `list_append` - list manipulation for crossover
- `list_nth` - element access for selection
- `random_int` - stochastic operations (mutation, selection)
