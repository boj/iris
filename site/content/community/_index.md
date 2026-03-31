---
title: "Community"
description: "Get involved with the IRIS community."
layout: "single"
---

IRIS is an open-source project. Here's how to get involved.

## Source Code {#source}

IRIS is developed on GitHub. The repository contains the self-hosted IRIS compiler and evaluator, the IRIS program library, and the test suite.

**Repository:** [github.com/boj/iris](https://github.com/boj/iris)

## Contributing {#contributing}

Contributions are welcome. The best way to start:

1. **Read the code.** Start with the [Architecture guide](/learn/architecture/) to understand the four-layer stack.
2. **Run the tests.** `iris-stage0 test` runs the full test suite.
3. **Pick an area.** The codebase is organized into `.iris` program modules -- find the one that interests you.
4. **Open a PR.** Fork, branch, and submit. Every PR should include tests.

### Areas of Interest {#areas}

- **Standard library** -- new modules, more operations, better ergonomics
- **Compiler passes** -- optimization improvements in the 10-pass pipeline
- **Evolution strategies** -- new selection, mutation, or crossover operators
- **Editor support** -- LSP features, syntax highlighting for more editors
- **Documentation** -- examples, tutorials, and guides
- **Benchmarks** -- new benchmark implementations, performance improvements

## Reporting Issues {#issues}

Found a bug? Have a feature request? [Open an issue](https://github.com/boj/iris/issues) on GitHub.

## License {#license}

IRIS is licensed under [AGPL-3.0-or-later](https://github.com/boj/iris/blob/main/LICENSE). Commercial licensing is available -- contact Brian Jones (bojo@bojo.wtf) for details.
