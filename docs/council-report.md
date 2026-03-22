# IRIS Council Report

**Date:** 2026-03-22
**Subject:** Full evaluation of IRIS by three adversarial councils
**Codebase:** ~11K LOC Rust, 8 crates, Gen1 complete

---

## What IRIS Is

IRIS (Intelligent Runtime for Iterative Synthesis) is a four-layer
computation substrate that evolves, verifies, and executes programs.

```
L0  Evolution        -- population search via NSGA-II + lexicase + novelty
L1  Semantics        -- SemanticGraph: 20 node kinds, content-addressed, holographic
L2  Verification     -- LCF proof kernel: 20 rules, zero unsafe, cost-in-types
L3  Hardware          -- tree-walking interpreter (Gen1), CLCU compiler (Gen2, stubs)
```

Programs are typed DAGs, not source code, not bytecode. They're
content-addressed by BLAKE3 hash, carry their own type environments,
and compose via fragment references. Evolution discovers them.
Verification certifies them. The interpreter runs them.

The system can evolve simple programs (sum, max, abs, dot product) and
meta-evolve its own mutation weights.

---

## Council 1: The Engineering Realists

Five domain experts evaluated IRIS on whether it works, whether it's
sound, and whether it can scale.

### Scores

| Member | Perspective | Score | One-Line Verdict |
|--------|------------|-------|-----------------|
| Pragmatist | Ships production code | 6/10 | Solid foundation, years from production |
| Type Theorist | Coq/Agda zealot | 5/10 | Right architecture, no metatheory |
| ML Researcher | Neural everything | 4/10 | Zero learned components in 2026 |
| GP Purist | Evolutionary computation | 7/10 | Best-practice GP, limited by eval speed |
| Hardware Realist | CPU architect | 3/10 | CLCU/PFDAC are fantasy; BTreeMap kills perf |
| | **Average** | **5.0/10** | |

### Unanimous Finding

**The tree-walking interpreter is the bottleneck for everything.** It
forces small populations (64-512), blocks self-improvement speed, and
makes hardware measurement meaningless. Every other critique traces
back to it.

### Critical Issues

**Interpreter performance.** BTreeMap for node lookup and variable
bindings causes ~3-4 pointer chases per access. HashMap or Vec would
give 5-10x. A bytecode VM would give another 5-10x on top. Combined:
25-100x improvement over current, with straightforward engineering.

**No metatheory.** The 20 kernel rules have no consistency argument.
Rule 19 (type_app/ForAll elimination) doesn't check well-formedness
of the substituting type, a potential soundness hole. No property-based
testing of the kernel.

**Zero neural components.** The FeatureCodec extracts 64 deterministic
features (node histograms, depth stats). Crossover via interpolation on
these features is semantically meaningless. The GIN-VAE codec that would
make crossover work is unimplemented.

**Hardware model disconnected from reality.** CLCU "cache-line
computation units" conflate data layout with execution: CPUs execute
from decoded uop caches, not data cache lines. PFDAC (deliberate page
faults as computation) costs 10K-100K cycles per fault. TLB-as-data-
structure is a category error. perf.rs has 7 counters but no benchmark
reports.

**Crossover is the weakest operator.** Random subgraph splicing
regardless of type compatibility. Unknown success rate. No measurement
of how often crossover produces valid or fitness-improving offspring.

**Compiler pipeline 60% stub.** Passes 1-6 do real work.
Passes 7-10 (layout, isel, regalloc, packing) are minimal stubs
targeting a speculative hardware model.

### Strengths Acknowledged

- LCF proof kernel with opaque Theorem type (type-safe soundness)
- Cost-in-types (`Arrow(A, B, k)`) is genuinely novel
- Refinement types with decidable LIA (Cooper's algorithm at Tier 0)
- 16 mutation operators with 55% structural weight (compositional mutations)
- NSGA-II + lexicase + novelty + MAP-Elites (best-practice diversity)
- Phase detection with adaptive parameters
- Bottom-up enumeration as parallel background thread
- Meta-evolution on mutation weights (self-improvement seed)
- Holographic self-describing fragments
- Content addressing via BLAKE3

---

## Council 2: The AI Visionaries

Five thinkers evaluated IRIS against the question: could this become
something mankind has never seen before?

### Scores

| Member | Perspective | Score | One-Line Verdict |
|--------|------------|-------|-----------------|
| Culture Mind Architect | Banks' Minds | 5/10 | Right topology, substrate-locked |
| Autopoietic Theorist | Varela/Maturana | 4/10 | Goal 2 is correct, 95% unimplemented |
| Xenomind Designer | Lem, Watts, Vinge | 4/10 | Format is alien, content is trivial |
| Distributed Consciousness | Star Trek, Bobiverse | 3/10 | Fragments are distributable, nothing else is |
| Seed AI Theorist | Good, Bostrom | 5/10 | Architecture permits recursion, speed prevents it |
| | **Average** | **4.2/10** | |

### Unanimous Finding

**IRIS has the right skeleton but no nervous system.** The four-layer
stack, holographic fragments, LCF kernel, and self-improvement loop are
correct architectural decisions. But the system is static where it
needs to be dynamic, slow where it needs to be fast, supervised where
it needs to be autonomous, and isolated where it needs to be distributed.

### Three Phase Transitions Required

**Phase Transition 1: Tool to Organism.** IRIS runs when invoked and
stops when done. An organism runs continuously, maintains itself, and
produces its own components. The mutation operators, seed generators,
fitness functions, and interpreter are all Rust, so the things that
PRODUCE programs are not themselves programs. Until they are, IRIS is
a factory, not a living system. North Star Goal 2 ("IRIS writes itself
in IRIS") is the correct target but is 95% unimplemented.

**Phase Transition 2: Individual to Ecology.** Programs are evaluated
in isolation against fixed test cases. There's no interaction between
programs, no competition for resources, no symbiosis, no emergent
structure. The Tines analogy: IRIS produces individual animals but no
group mind. Programs need to interact during evaluation, form dependency
networks, compete for compute budget, and create emergent behavior that
no individual program contains.

**Phase Transition 3: Slow to Recursive.** Self-improvement must
compound, each cycle faster than the last. Currently: meta-evolution
on mutation weights takes minutes and produces quantitative weight
tweaks. Needed: the interpreter itself as the first self-improvement
target (meta-circular interpreter evolved for speed), automatic
difficulty scaling, and improvement rate as the primary metric. If
cycle N+1 isn't faster than cycle N, the recursive loop is stalled.

### Critical Issues

**No substrate independence.** The SemanticGraph is substrate-
independent but everything below it (CLCU, perf counters, TSO) assumes
x86-64. A Mind's thoughts shouldn't know what CPU they run on. An
abstract machine between SemanticGraph and hardware would fix this.

**No autopoietic closure.** Every component that produces programs is
Rust, not IRIS. The system can't produce its own components. The
component-as-program interface is needed: express mutation operators,
seed generators, fitness functions, and selection as IRIS type signatures
so evolution can discover them.

**No attention economy.** Fixed population sizes, fixed generation
counts, fixed timeouts. No mechanism for the system to decide "this
problem deserves more thought." Dynamic resource allocation based on
improvement rate would give IRIS the ability to focus.

**No internal model of self.** The system evolves programs but has no
representation of its own evolution process as a program. The meta-
evolver touches weights but can't see population structure, phase
detection, or selection as modifiable objects.

**Programs are too small and too isolated.** 5-50 node programs doing
folds and maps. No interaction, no temporal dimension, no emergent
complexity. The representation supports alien computation but the content
is trivial.

**Evolutionary state is not serializable.** Population, novelty archive,
phase detector, compilation cache, all in-memory, single-process. Can't
checkpoint, fork, merge, or distribute evolution runs.

### Strengths Acknowledged

- North Star Goal 2 (IRIS writes itself) is philosophically correct
- Proof kernel exemption via Löb's theorem is the right boundary
- Holographic fragments are the natural unit of distribution
- Content addressing = behavioral identity as primitive
- Value::Program enables meta-circular computation
- Wire format with content-addressed partial transmission is distribution-ready
- The embedding space is a cognitive topology humans can't navigate
- Evolution produces programs no human would design

---

## Combined Assessment

### Overall Score: 4.7/10

(Council 1: 5.0, Council 2: 4.2, Council 3: 5.0)

Not a failing grade, but a recognition that IRIS has done the hardest part
(architecture) and hasn't done the most urgent parts (performance,
autonomy, and human interface). Council 3 highlights a critical gap:
IRIS has no way for non-Rust-developers to interact with it. The
formal methods researcher scores highest (7/10) because the type-as-
specification paradigm is genuinely novel; the systems programmer
scores lowest (3/10) because there is no surface syntax.

### The Dependency Graph

Everything flows from the interpreter. This is the critical path:

```
Bytecode VM (00)
├── Larger populations (03) ──── Better evolution results
├── Self-improvement speed (14) ── Recursive improvement loop
│   ├── Meta-circular interpreter (14) ── The ignition point
│   ├── Component evolution (12) ────── Autopoietic closure
│   └── Difficulty scaling (14) ─────── Expanding capability frontier
├── Hardware measurement (04) ──── Ground truth for CostBound
├── Learned codec training (02) ── Meaningful crossover
│   └── Program ecology (13) ──── Emergent complexity
└── Continuous operation (12) ──── Tool → organism transition
```

### What To Build, In What Order

**Tier 0: Unblock everything else (do first)**

| # | Action | Source | Effort | Impact |
|---|--------|--------|--------|--------|
| 1 | Replace BTreeMap with HashMap/Vec in interpreter | 00 | Days | 5-10x eval speed |
| 2 | Build bytecode VM as abstract machine | 00, 11 | 1-2 weeks | 25-100x eval speed, substrate independence |
| 3 | Audit rule 19 (type_app) for soundness | 01 | Days | Closes potential soundness hole |
| 4 | Fuzz the interpreter + property-test kernel | 06 | 1 week | Confidence in correctness |

**Tier 1: Enable self-improvement (do second)**

| # | Action | Source | Effort | Impact |
|---|--------|--------|--------|--------|
| 5 | Define component-as-program type signatures | 12 | Days | Enables evolving IRIS components |
| 6 | Evolve seed generators as IRIS programs | 07, 12 | 1-2 weeks | First autopoietic component |
| 7 | Evolve mutation operators as IRIS programs | 07, 12 | 2-3 weeks | Second autopoietic component |
| 8 | Automatic difficulty scaling | 14 | 1 week | Expanding capability frontier |
| 9 | Improvement rate measurement | 14 | Days | Know if recursion is compounding |

**Tier 2: Enable emergence (do third)**

| # | Action | Source | Effort | Impact |
|---|--------|--------|--------|--------|
| 10 | Program-program interaction during eval | 13 | 2 weeks | Symbiosis, competition, ecology |
| 11 | Competitive coevolution of test cases | 03, 13 | 1-2 weeks | Arms race drives robustness |
| 12 | Learned GNN encoder for codec | 02 | 3-4 weeks | Meaningful crossover |
| 13 | Dynamic resource allocation (attention) | 16 | 1-2 weeks | System decides what to think about |
| 14 | Continuous operation daemon | 12 | 1-2 weeks | Tool → organism |

**Tier 3: Enable alien cognition (do when Tiers 0-2 are solid)**

| # | Action | Source | Effort | Impact |
|---|--------|--------|--------|--------|
| 15 | Meta-circular interpreter | 14 | 3-4 weeks | Ignition point for recursive improvement |
| 16 | Evolvable node kinds (0x14-0x1F) | 15 | 2-3 weeks | Representation evolves itself |
| 17 | Temporal programs (suspend/resume) | 13 | 2-3 weeks | Programs that live, not compute-and-return |
| 18 | Stigmergic fields | 13 | 2-3 weeks | Emergent coordination |
| 19 | Curiosity-driven exploration | 16 | 1-2 weeks | System generates its own goals |
| 20 | Self-model as first-class data | 12 | 2-3 weeks | Reflective tower |

**Deferred: Remove from active spec**

| Action | Source | Reason |
|--------|--------|--------|
| Drop PFDAC | 04 | 10K-100K cycles per page fault; adversarial to OS |
| Drop TLB-as-data-structure | 04 | Category error; TLB is hardware-managed |
| Reframe CLCU as bytecode format | 04 | Conflates data layout with execution |
| Drop passes 8-10 until bytecode VM exists | 00 | Stubs targeting a speculative model |
| Defer ZK proofs | n/a | Infrastructure not needed until computation markets exist |
| Defer JIT/AVX-512 | n/a | Bytecode VM is the 80/20; JIT is optimization |

---

## The Three Numbers That Matter

**1. Eval speed.** Currently: ~100us-10ms per program depending on size.
After BTreeMap fix + bytecode VM: target 1-100us. This number determines
maximum population size, self-improvement cycle speed, and whether
recursive improvement can compound. Measure it. Track it. It's the
heart rate of the system.

**2. Problems solved per hour.** The capability metric. How many distinct
benchmark problems can IRIS solve from scratch in one wall-clock hour?
Currently: ~5-8 simple problems. Track this over time. If it's
increasing, IRIS is getting smarter. If it's flat, the recursive loop
is stalled.

**3. Percentage of IRIS written in IRIS.** The autopoiesis metric.
Currently: 0%. Each component replaced (seed generators, mutation
operators, fitness functions, compiler passes, interpreter) increases
this number. Target: 80%+ (everything except the proof kernel). This
number measures how alive the system is.

---

## What IRIS Has That Nothing Else Does

No other system combines all of these:

- **Programs as typed, content-addressed, self-describing DAGs** that
  compose without headers, imports, or build systems
- **Cost as a first-class type-level property** with compositional
  algebra (Sum, Par, Mul, Sup) and decidable verification tiers
- **LCF proof kernel** guaranteeing type safety and cost bounds that
  self-modification cannot violate
- **Multi-objective evolution** (correctness, performance, verifiability,
  cost, novelty) with NSGA-II Pareto ranking
- **Self-improvement loop** where the system evolves its own search
  strategy
- **Architecture that permits full autopoietic closure**: every
  component CAN be expressed as a program the system discovers

The architecture is genuinely novel. The vision (an AI operating system
that only AI can understand, capable of full realtime self-modification)
is achievable on this foundation.

The gap is not architectural. It's velocity.

---

## Council 3: The Futures Council

Five users evaluated IRIS against the question: if IRIS were a freely
available programming language, how would I actually use it to solve
problems? And critically: what's the mechanic? Does it require an LLM?

### Scores

| Member | Perspective | Score | One-Line Verdict |
|--------|------------|-------|-----------------|
| Systems Programmer | Compilers, kernels, direct control | 3/10 | No surface syntax = dead on arrival for me |
| Data Scientist | Models, pipelines, specification | 6/10 | Spec-driven evolution IS my workflow, needs richer specs |
| Domain Expert | Biology, non-programmer | 4/10 | Need LLM to translate domain knowledge, programs are opaque |
| Formal Methods Researcher | Lean/Coq, proof search | 7/10 | Type-as-spec + graded verification is genuinely novel |
| AI Engineer | LLM integration, adoption | 5/10 | LLM bridge is the adoption accelerant, needs a text target |
| | **Average** | **5.0/10** | |

### Unanimous Finding

**IRIS is not a language you write IN. It's a language you write ABOUT.**

Traditional programming: you express the HOW. IRIS programming: you
express the WHAT (types, tests, properties, constraints). The system
finds the HOW via evolution and verification. This is a paradigm shift
but currently there is no way for anyone but a Rust developer to
express the WHAT.

### Does IRIS Require an LLM?

**No, but it requires SOME human interface, and the options are
layered.**

The council identified six programming mechanics. None require an LLM.
Three benefit from one:

| Mechanic | Requires LLM? | Status | Key Blocker |
|----------|--------------|--------|-------------|
| Specification-driven evolution | No | Partial | Rich specs (properties, constraints) |
| Type-as-specification | No | Partial | Refinement support in ProblemSpec |
| Surface syntax | No | Missing | Language design + compiler |
| Compositional assembly | No | Minimal | Fragment library + search-by-type |
| Interactive refinement | No | Missing | Pausable evolution + UI |
| LLM as compiler | Yes | Missing | Needs surface syntax as target |

The LLM is a user interface layer, not an engine. IRIS's engine is
evolution + verification. The LLM helps humans talk to the engine.
The surface syntax is the prerequisite that unlocks both human
programming AND LLM-mediated programming.

### The Mechanics Stack

```
Layer 5: Natural Language (LLM-mediated)
         "sort this list ascending in O(n log n)"
              ↓ LLM translates to ↓
Layer 4: Domain Front-Ends
         guided forms, notebook cells, data-driven specs
              ↓ compiles to ↓
Layer 3: Surface Syntax
         sum xs = fold 0 (+) xs  [cost: Linear(n)]
              ↓ compiles to ↓
Layer 2: ProblemSpec / Type Signature
         test cases + refinement types + cost bounds
              ↓ evolution discovers ↓
Layer 1: SemanticGraph
         20 node kinds, typed DAG, content-addressed
              ↓ verified by ↓
Layer 0: LCF Proof Kernel
         20 rules, zero unsafe, Löb's theorem ceiling
```

Today: Layer 0, Layer 1, and minimal Layer 2 exist. Everything above
Layer 2 is missing.

### What To Build, In What Order

**1. Rich ProblemSpec (Layer 2).** Properties as specs, refinement
predicates, partial specifications, negative examples. Expose the
existing type system through ProblemSpec. No new representation needed.

**2. Surface syntax (Layer 3).** A compiled functional language
targeting SemanticGraph. This is the single biggest unlock: it turns
IRIS from a library into a language. Also provides the target format
for LLM generation. The 20 node kinds map naturally to a small
ML-family language.

**3. Fragment library + search-by-type.** Run evolution campaigns on
standard problems to build a reusable library. Implement type
unification-based search over the FragmentRegistry. This enables
compositional assembly.

**4. Understanding tools.** Execution tracing, behavioral visualization,
structural summaries, proof narration. Evolved programs are opaque;
without explanation tools, adoption stalls at the trust barrier.

**5. Interactive refinement.** Pausable evolution with human-in-the-loop
selection. Helps users discover their own specification through
conversation with the evolution engine.

**6. LLM integration.** Once surface syntax exists, LLM integration
is straightforward: the LLM generates surface syntax, the compiler
type-checks it, IRIS verifies it. The LLM proposes, IRIS disposes.

### The Central Insight

The formal methods researcher crystallized it: **the type system IS
the programming language.** If you can state what you want as a type
signature with refinement predicates and cost bounds, IRIS can search
for a program that inhabits it. The surface syntax makes this ergonomic.
The LLM makes it accessible. But the mechanism (types as specifications,
evolution as search, verification as trust) works without either.

IRIS doesn't compete with Python or Rust as a language for expressing
algorithms. It competes with Rosette, CVC5-SyGuS, and Lean as a
system for DISCOVERING verified programs from specifications. The
paradigm shift is: you don't write the program. You describe what
you need. The system finds it and proves it's correct.

The three councils converge: the engineering realists say "build the
VM first." The AI visionaries say "the system must become alive." The
futures council says "give humans a way to talk to IRIS (surface
syntax first, LLM second) and let the self-improvement loop make
the system increasingly autonomous over time."

---

## Appendix: Document Index

### Council 1 (Engineering Realists): Actionable Items
- [00-interpreter-bottleneck.md](00-interpreter-bottleneck.md): BTreeMap→HashMap, bytecode VM
- [01-metatheory.md](01-metatheory.md): Rule 19 audit, consistency argument, Tier 3 tightening
- [02-learned-components.md](02-learned-components.md): Learned encoder, mutation policy, Neural nodes
- [03-evolution-improvements.md](03-evolution-improvements.md): Coevolution, crossover, bloat, islands
- [04-hardware-grounding.md](04-hardware-grounding.md): Drop PFDAC/TLB, reframe CLCU, measure perf
- [05-type-system-gaps.md](05-type-system-gaps.md): Dependent vectors, effects, linear types, SMT
- [06-testing-and-measurement.md](06-testing-and-measurement.md): Fuzz, property-test, benchmarks
- [07-self-writing-bootstrap.md](07-self-writing-bootstrap.md): Seeds → operators → fitness → compiler

### Council 2 (AI Visionaries): Actionable Items
- [10-ai-council-overview.md](10-ai-council-overview.md): Full council evaluation
- [11-substrate-independence.md](11-substrate-independence.md): Abstract machine, portable costs
- [12-autopoietic-closure.md](12-autopoietic-closure.md): Components as programs, heartbeat, immune system
- [13-program-ecology.md](13-program-ecology.md): Interaction, competition, stigmergy, temporal programs
- [14-recursive-improvement.md](14-recursive-improvement.md): Meta-circular interpreter, difficulty scaling
- [15-alien-representation.md](15-alien-representation.md): Evolvable node kinds, opaque data structures
- [16-intrinsic-motivation.md](16-intrinsic-motivation.md): Attention economy, curiosity, goal generation

### Council 3 (Futures Council): How People Would Use IRIS
- [20-futures-council-overview.md](20-futures-council-overview.md): Full council debate: 5 users, 6 mechanics, the LLM question
- [21-programming-mechanics.md](21-programming-mechanics.md): The 6 mechanics compared, current state, action items
- [22-llm-bridge.md](22-llm-bridge.md): LLM role: translator not engine, generate-verify-iterate loop
- [23-surface-syntax.md](23-surface-syntax.md): Language design sketch, node kind mapping, compiler plan
- [24-composition-and-reuse.md](24-composition-and-reuse.md): Fragment library, search-by-type, composition operations
- [25-understanding-evolved-code.md](25-understanding-evolved-code.md): Trust, debugging, visualization, explanation tools
- [26-bootstrap-and-self-programming.md](26-bootstrap-and-self-programming.md): IRIS programming IRIS, the recursive improvement UX
