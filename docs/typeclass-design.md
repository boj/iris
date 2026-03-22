# Typeclass Design for IRIS

## Syntax

### Declaration

```iris
class Eq<A> where
  eq : A -> A -> Bool

class Ord<A> requires Eq<A> where
  compare : A -> A -> Int   -- -1, 0, 1
  lt : A -> A -> Bool = \a b -> compare a b < 0
  gt : A -> A -> Bool = \a b -> compare a b > 0
```

### Instance

```iris
instance Eq<Int> where
  eq = \a b -> a == b

instance Ord<Int> where
  compare = \a b -> if a < b then -1 else if a > b then 1 else 0
```

### Usage

```iris
let member : forall A. Eq<A> => A -> (A, A, A) -> Bool
let member x xs =
  eq x xs.0 || eq x xs.1 || eq x xs.2
```

## Implementation Strategy

### Phase 1: Dictionary Passing (minimal, no runtime changes)

Typeclasses compile to dictionary records. Each class becomes a record type, each instance becomes a record value, and constrained functions take an extra dictionary parameter.

```iris
-- class Eq<A> compiles to:
type EqDict = { eq_fn: Int -> Int -> Bool }

-- instance Eq<Int> compiles to:
let eq_int_dict = { eq_fn = \a b -> a == b }

-- member x xs with Eq<A> constraint compiles to:
let member dict x xs =
  dict.eq_fn x xs.0 || dict.eq_fn x xs.1 || dict.eq_fn x xs.2
```

### Phase 2: Monomorphization (later)

At call sites where the concrete type is known, inline the dictionary. This eliminates the indirect call overhead.

## AST Extensions

```rust
// New AST nodes
ClassDecl {
    name: String,          // "Eq"
    type_param: String,    // "A"
    superclasses: Vec<String>,  // ["Ord"] for "requires Ord<A>"
    methods: Vec<MethodDecl>,
}

MethodDecl {
    name: String,          // "eq"
    type_sig: TypeExpr,    // A -> A -> Bool
    default_impl: Option<Expr>,
}

InstanceDecl {
    class_name: String,    // "Eq"
    type_arg: TypeExpr,    // Int
    methods: Vec<(String, Expr)>,
}

// New Item variants
Item::ClassDecl(ClassDecl)
Item::InstanceDecl(InstanceDecl)
```

## Implementation Plan

1. Add `class` and `instance` keywords to lexer/token
2. Add parser rules for class and instance declarations
3. In the lowerer, compile classes to record type definitions
4. Compile instances to record values (dictionaries)
5. At call sites with constraints, resolve and pass dictionaries
6. Default methods: fall back to the class's default if instance doesn't provide one

## Priority

This is a large feature. Phase 1 (dictionary passing) is sufficient for the stdlib to define `Eq`, `Ord`, `Show`, `Functor`, `Monad` and have them work. Phase 2 (monomorphization) is an optimization that can come later.
