# Calculator

An arithmetic expression evaluator supporting +, -, *, / and nested expressions.

## How it works

Expressions are represented as tagged tuple ASTs:
- `(0, value)` - number literal
- `(1, left, right)` - addition
- `(2, left, right)` - subtraction
- `(3, left, right)` - multiplication
- `(4, left, right)` - division (returns 0 for division by zero)

The `eval_ast` function recursively evaluates the tree. The `eval_simple`
function handles flat `a op b` expressions.

## Running

```bash
iris run examples/calculator/calculator.iris
iris run examples/calculator/calculator-test.iris
```

## Primitives used

- `if-then-else` - conditional dispatch on AST tags
- `list_nth` - extract fields from tuple-based AST nodes
- Arithmetic operators: `+`, `-`, `*`, `/`
