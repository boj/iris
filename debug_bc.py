#!/usr/bin/env python3
"""
IRIS bytecode interpreter for debugging native VM crashes.

Executes pipeline stage bytecodes with tracing to find where a non-tuple value
appears where TUPLE_GET/TUPLE_GET_DYN expects a tuple pointer.

Usage:
    python3 debug_bc.py                    # stage 0 (tokenizer)
    python3 debug_bc.py --stage 0          # stage 0 (tokenizer)
    python3 debug_bc.py --stage 1          # stage 1 (parser) — runs tok first
    python3 debug_bc.py --stage 2          # stage 2 (AST compiler) — runs tok+par first
    python3 debug_bc.py --stage 1 --trace  # with full opcode tracing
    python3 debug_bc.py <file>             # read bytecodes from file (stage 0 mode)
"""

import sys
import os

# ---------------------------------------------------------------------------
# Opcode names
# ---------------------------------------------------------------------------
OP_NAMES = {
    0: "HALT", 1: "PUSH", 2: "ADD", 3: "SUB", 4: "MUL", 5: "DIV",
    6: "MOD", 7: "NEG", 8: "EQ", 9: "LT", 10: "GT", 11: "NE",
    12: "LE", 13: "GE", 14: "LOAD", 15: "STORE", 16: "JMP", 17: "JZ",
    18: "MAKE_TUPLE", 19: "TUPLE_GET", 20: "???20", 21: "TUPLE_LEN",
    22: "LIST_APPEND", 23: "BITAND", 24: "SHR", 25: "FOLD_BEGIN",
    26: "FOLD_END", 27: "LIST_RANGE", 28: "PRIM_CALL", 29: "PUSH_STR_PTR",
    30: "STR_LEN", 31: "CHAR_AT", 32: "STR_CONCAT", 33: "STR_SLICE",
    34: "LIST_CONCAT", 35: "TUPLE_GET_DYN", 36: "???36", 37: "???37",
    38: "???38", 39: "FILE_READ", 40: "DEBUG_PRINT",
}

# ---------------------------------------------------------------------------
# Parse bytecodes from IRIS output: (count, (bc0, bc1, ...))
# ---------------------------------------------------------------------------
def parse_bytecodes(text):
    """Parse (count, (v0, v1, ...)) output from save_pipeline_bc.iris"""
    # Join all lines and normalize whitespace
    text = " ".join(text.split()).strip()

    # Find the outer tuple: (count, (bc...))
    # Remove outer parens
    if text.startswith("(") and text.endswith(")"):
        text = text[1:-1]

    # Find the first comma that separates count from the inner tuple
    depth = 0
    split_pos = -1
    for i, ch in enumerate(text):
        if ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
        elif ch == "," and depth == 0:
            split_pos = i
            break

    if split_pos < 0:
        raise ValueError("Cannot parse bytecode output")

    count_str = text[:split_pos].strip()
    rest = text[split_pos + 1:].strip()

    count = int(count_str)

    # rest is "(bc0, bc1, ...)"
    if rest.startswith("(") and rest.endswith(")"):
        rest = rest[1:-1]

    # Parse comma-separated values (integers or () for unit)
    bc = []
    for token in rest.split(","):
        token = token.strip()
        if token:
            if token == "()":
                bc.append(())  # unit value
            else:
                bc.append(int(token))

    print(f"Parsed {len(bc)} bytecodes (header says {count})", file=sys.stderr)
    return bc


# ---------------------------------------------------------------------------
# Fold state
# ---------------------------------------------------------------------------
class FoldState:
    def __init__(self, limit, loop_pc, counter=0, parent=None):
        self.limit = limit
        self.loop_pc = loop_pc
        self.counter = counter
        self.parent = parent

    def __repr__(self):
        return f"Fold(limit={self.limit}, counter={self.counter}, loop_pc={self.loop_pc})"


# ---------------------------------------------------------------------------
# Value helpers
# ---------------------------------------------------------------------------
def is_tuple(v):
    return isinstance(v, (tuple, list))

def is_string(v):
    return isinstance(v, str)

def tuple_len(v):
    if v is None or v == 0:
        return 0
    if is_tuple(v):
        return len(v)
    return 0

def tuple_get(v, idx):
    if v is None or v == 0:
        return 0
    if is_tuple(v):
        if 0 <= idx < len(v):
            return v[idx]
        return 0
    # NOT a tuple -- this is the bug we're looking for
    return 0

def as_tuple(v):
    """Convert value to a tuple for concatenation etc."""
    if v is None or v == 0:
        return ()
    if is_tuple(v):
        return tuple(v)
    return (v,)

def val_repr(v, max_depth=3, max_items=8):
    """Compact representation of a value for tracing."""
    if v is None:
        return "None"
    if isinstance(v, int):
        return str(v)
    if isinstance(v, str):
        if len(v) > 40:
            return f'"{v[:37]}..."[{len(v)}]'
        return f'"{v}"'
    if is_tuple(v):
        if max_depth <= 0:
            return f"(...)[{len(v)}]"
        items = []
        for i, x in enumerate(v[:max_items]):
            items.append(val_repr(x, max_depth - 1, max_items))
        if len(v) > max_items:
            items.append(f"...+{len(v) - max_items}")
        return f"({', '.join(items)})"
    return repr(v)


# ---------------------------------------------------------------------------
# VM
# ---------------------------------------------------------------------------
def run_vm(bc, input_val, max_steps=1_000_000, trace_all=False, trace_file=None,
           extra_inputs=None):
    """
    Execute IRIS bytecodes.

    bc: list of int bytecodes
    input_val: the primary input value (pushed onto stack AND stored in locals[0])
    extra_inputs: dict of {slot: value} for additional inputs (e.g. {1: source_str})
                  These are stored in locals only, NOT pushed onto the stack.
                  This matches native VM behavior: rsi→stack+locals[0], rdx→locals[1] only.
    max_steps: abort after this many steps
    trace_all: if True, print every opcode execution
    trace_file: file object for trace output (default stderr)
    """
    if trace_file is None:
        trace_file = sys.stderr

    stack = []       # value stack (append/pop from end = top)
    locals_ = {}     # slot -> value
    fold_stack = []  # stack of FoldState

    # Store primary input as locals[0] and push onto stack
    locals_[0] = input_val
    stack.append(input_val)

    # Store extra inputs in locals only (not pushed onto stack)
    if extra_inputs:
        for slot, val in extra_inputs.items():
            locals_[slot] = val

    pc = 0
    step = 0
    history = []  # (pc, opcode, extra_info)

    def push(v):
        stack.append(v)

    def pop():
        if not stack:
            raise RuntimeError(f"Stack underflow at PC={pc}, step={step}")
        return stack.pop()

    def peek():
        if not stack:
            return "<empty>"
        return stack[-1]

    def dump_state(reason, extra=""):
        """Dump debugging state."""
        print(f"\n{'='*72}", file=trace_file)
        print(f"STOP: {reason}", file=trace_file)
        if extra:
            print(f"  {extra}", file=trace_file)
        print(f"  PC={pc}  Step={step}", file=trace_file)
        print(f"\n  Last 30 opcodes:", file=trace_file)
        for h_pc, h_op, h_info in history[-30:]:
            name = OP_NAMES.get(h_op, f"???{h_op}")
            info_str = f"  {h_info}" if h_info else ""
            print(f"    PC={h_pc:6d}  {name}({h_op}){info_str}", file=trace_file)
        print(f"\n  Stack (top 15, len={len(stack)}):", file=trace_file)
        for i, v in enumerate(reversed(stack[-15:])):
            tag = "INT" if isinstance(v, int) else "STR" if isinstance(v, str) else f"TUPLE[{len(v)}]" if is_tuple(v) else type(v).__name__
            print(f"    [{i:2d}] {tag:10s}  {val_repr(v, max_depth=2, max_items=5)}", file=trace_file)
        if fold_stack:
            print(f"\n  Fold stack (depth={len(fold_stack)}):", file=trace_file)
            for i, fs in enumerate(fold_stack):
                print(f"    [{i}] {fs}", file=trace_file)
        # Print relevant locals
        print(f"\n  Locals (non-zero):", file=trace_file)
        for slot in sorted(locals_.keys()):
            v = locals_[slot]
            if v != 0:
                print(f"    [{slot:4d}] {val_repr(v, max_depth=1, max_items=4)}", file=trace_file)
        print(f"{'='*72}\n", file=trace_file)

    while pc < len(bc):
        if step >= max_steps:
            dump_state(f"Max steps ({max_steps}) exceeded")
            return None

        op = bc[pc]

        # --- Tracing for TUPLE_GET / TUPLE_GET_DYN ---
        if op == 19:  # TUPLE_GET
            # The tuple is TOS
            if stack:
                tos = stack[-1]
                if isinstance(tos, int) and tos != 0:
                    dump_state(
                        f"TUPLE_GET on non-tuple value",
                        f"TOS = {tos} (int), expected tuple. idx={bc[pc+1] if pc+1 < len(bc) else '?'}"
                    )
                    return None
        elif op == 35:  # TUPLE_GET_DYN
            # Stack: ..., tuple, index  (index is TOS)
            if len(stack) >= 2:
                tup_val = stack[-2]
                if isinstance(tup_val, int) and tup_val != 0:
                    dump_state(
                        f"TUPLE_GET_DYN on non-tuple value",
                        f"tuple_val = {tup_val} (int), index TOS = {stack[-1]}"
                    )
                    return None

        # Record history
        extra_info = ""
        if op == 1 and pc + 1 < len(bc):
            extra_info = f"val={bc[pc+1]}"
        elif op == 14 and pc + 1 < len(bc):
            slot = bc[pc + 1]
            extra_info = f"slot={slot} -> {val_repr(locals_.get(slot, 0), max_depth=1, max_items=3)}"
        elif op == 15 and pc + 1 < len(bc):
            extra_info = f"slot={bc[pc+1]} <- {val_repr(peek(), max_depth=1, max_items=3)}"
        elif op == 18 and pc + 1 < len(bc):
            extra_info = f"n={bc[pc+1]}"
        elif op == 19 and pc + 1 < len(bc):
            extra_info = f"idx={bc[pc+1]} from {val_repr(peek(), max_depth=1, max_items=3)}"
        elif op == 16 and pc + 1 < len(bc):
            extra_info = f"offset={bc[pc+1]} -> PC={pc + bc[pc+1]}"
        elif op == 17 and pc + 1 < len(bc):
            extra_info = f"offset={bc[pc+1]}, cond={val_repr(peek(), max_depth=0)}"
        elif op == 25 and pc + 1 < len(bc):
            extra_info = f"body_len={bc[pc+1]}"

        history.append((pc, op, extra_info))

        if trace_all:
            name = OP_NAMES.get(op, f"???{op}")
            info_str = f"  {extra_info}" if extra_info else ""
            slen = len(stack)
            print(f"  [{step:7d}] PC={pc:6d}  {name}({op}){info_str}  stk={slen}", file=trace_file)

        # --- Dispatch ---
        if op == 0:  # HALT
            result = pop() if stack else 0
            print(f"\nHALT at PC={pc}, step={step}", file=trace_file)
            print(f"Result type: {'tuple' if is_tuple(result) else 'str' if is_string(result) else 'int'}", file=trace_file)
            if is_tuple(result):
                print(f"Result length: {len(result)}", file=trace_file)
                print(f"Result preview: {val_repr(result, max_depth=2, max_items=10)}", file=trace_file)
            else:
                print(f"Result: {val_repr(result, max_depth=2)}", file=trace_file)
            return result

        elif op == 1:  # PUSH(val)
            val = bc[pc + 1]
            push(val)
            pc += 2

        elif op == 2:  # ADD
            a = pop(); b = pop()
            push(b + a)
            pc += 1

        elif op == 3:  # SUB
            a = pop(); b = pop()
            push(b - a)
            pc += 1

        elif op == 4:  # MUL
            a = pop(); b = pop()
            push(b * a)
            pc += 1

        elif op == 5:  # DIV
            a = pop(); b = pop()
            push(b // a if a != 0 else 0)
            pc += 1

        elif op == 6:  # MOD
            a = pop(); b = pop()
            push(b % a if a != 0 else 0)
            pc += 1

        elif op == 8:  # EQ
            a = pop(); b = pop()
            push(1 if b == a else 0)
            pc += 1

        elif op == 9:  # LT
            a = pop(); b = pop()
            push(1 if b < a else 0)
            pc += 1

        elif op == 10:  # GT
            a = pop(); b = pop()
            push(1 if b > a else 0)
            pc += 1

        elif op == 11:  # NE
            a = pop(); b = pop()
            push(1 if b != a else 0)
            pc += 1

        elif op == 12:  # LE
            a = pop(); b = pop()
            push(1 if b <= a else 0)
            pc += 1

        elif op == 13:  # GE
            a = pop(); b = pop()
            push(1 if b >= a else 0)
            pc += 1

        elif op == 14:  # LOAD(slot)
            slot = bc[pc + 1]
            push(locals_.get(slot, 0))
            pc += 2

        elif op == 15:  # STORE(slot)
            slot = bc[pc + 1]
            locals_[slot] = pop()
            pc += 2

        elif op == 16:  # JMP(offset) - relative
            offset = bc[pc + 1]
            pc += offset

        elif op == 17:  # JZ(offset) - relative
            val = pop()
            if val == 0:
                pc += bc[pc + 1]
            else:
                pc += 2

        elif op == 18:  # MAKE_TUPLE(n)
            n = bc[pc + 1]
            if n == 0:
                push(())
            else:
                # Pop n values; first-pushed = element 0
                # Since stack is LIFO, last popped = element 0
                vals = []
                for _ in range(n):
                    vals.append(pop())
                vals.reverse()
                push(tuple(vals))
            pc += 2

        elif op == 19:  # TUPLE_GET(idx)
            idx = bc[pc + 1]
            t = pop()
            if t is None or t == 0:
                push(0)
            elif is_tuple(t):
                if 0 <= idx < len(t):
                    push(t[idx])
                else:
                    push(0)
            elif is_string(t):
                # Some bytecodes treat strings as tuples of chars?
                push(0)
            else:
                # This is the bug case -- integer where tuple expected
                push(0)
            pc += 2

        elif op == 21:  # TUPLE_LEN
            t = pop()
            if t is None or t == 0:
                push(0)
            elif is_tuple(t):
                push(len(t))
            elif is_string(t):
                push(len(t))
            else:
                push(0)
            pc += 1

        elif op == 22:  # LIST_APPEND
            val = pop()
            t = pop()
            if t is None or t == 0:
                push((val,))
            elif is_tuple(t):
                push(tuple(t) + (val,))
            else:
                push((t, val))
            pc += 1

        elif op == 23:  # BITAND
            a = pop(); b = pop()
            if isinstance(a, int) and isinstance(b, int):
                push(b & a)
            else:
                push(0)
            pc += 1

        elif op == 24:  # SHR
            a = pop(); b = pop()
            if isinstance(a, int) and isinstance(b, int):
                push(b >> a)
            else:
                push(0)
            pc += 1

        elif op == 25:  # FOLD_BEGIN(body_len)
            body_len = bc[pc + 1]
            limit = pop()
            acc = pop()

            fs = FoldState(
                limit=limit,
                loop_pc=pc + 2,
                counter=0,
                parent=fold_stack[-1] if fold_stack else None,
            )
            fold_stack.append(fs)

            push(acc)        # accumulator
            push(0)          # counter = 0
            pc += 2

        elif op == 26:  # FOLD_END
            # The fold body leaves exactly ONE value on the stack: the new accumulator
            new_acc = pop()

            if not fold_stack:
                dump_state("FOLD_END with empty fold stack")
                return None

            fs = fold_stack[-1]
            fs.counter += 1

            if isinstance(fs.counter, int) and isinstance(fs.limit, int) and fs.counter < fs.limit:
                # Continue loop: push new_acc and counter, jump to loop start
                push(new_acc)
                push(fs.counter)
                pc = fs.loop_pc
            else:
                # Loop done: push result, pop fold state
                fold_stack.pop()
                push(new_acc)
                pc += 1

        elif op == 27:  # LIST_RANGE
            end = pop()
            start = pop()
            if isinstance(start, int) and isinstance(end, int) and end > start:
                push(tuple(range(start, end)))
            else:
                push(0)
            pc += 1

        elif op == 28:  # PRIM_CALL(op, nargs)
            prim_op = bc[pc + 1]
            nargs = bc[pc + 2]
            args = []
            for _ in range(nargs):
                args.append(pop())
            args.reverse()
            # Unknown prim calls just return 0
            push(0)
            pc += 3

        elif op == 29:  # PUSH_STR_PTR(val)
            val = bc[pc + 1]
            push(val)
            pc += 2

        elif op == 30:  # STR_LEN
            s = pop()
            if is_string(s):
                push(len(s))
            elif isinstance(s, int):
                push(0)
            else:
                push(0)
            pc += 1

        elif op == 31:  # CHAR_AT
            idx = pop()
            s = pop()
            if is_string(s) and isinstance(idx, int) and 0 <= idx < len(s):
                push(ord(s[idx]))
            else:
                push(0)
            pc += 1

        elif op == 32:  # STR_CONCAT
            s2 = pop()
            s1 = pop()
            if is_string(s1) and is_string(s2):
                push(s1 + s2)
            else:
                push(str(s1) + str(s2) if not is_string(s1) or not is_string(s2) else "")
            pc += 1

        elif op == 33:  # STR_SLICE
            end = pop()
            start = pop()
            s = pop()
            if is_string(s) and isinstance(start, int) and isinstance(end, int):
                push(s[start:end])
            else:
                push("")
            pc += 1

        elif op == 34:  # LIST_CONCAT
            t2 = pop()
            t1 = pop()
            push(as_tuple(t1) + as_tuple(t2))
            pc += 1

        elif op == 35:  # TUPLE_GET_DYN
            idx = pop()
            t = pop()
            if t is None or t == 0:
                push(0)
            elif is_tuple(t) and isinstance(idx, int):
                if 0 <= idx < len(t):
                    push(t[idx])
                else:
                    push(0)
            else:
                push(0)
            pc += 1

        elif op == 39:  # FILE_READ
            path = pop()
            if is_string(path):
                try:
                    with open(path, "r") as f:
                        push(f.read())
                except Exception as e:
                    print(f"FILE_READ error: {e}", file=sys.stderr)
                    push("")
            else:
                push("")
            pc += 1

        elif op == 40:  # DEBUG_PRINT
            val = pop()
            print(f"DEBUG_PRINT: {val_repr(val, max_depth=3, max_items=10)}", file=sys.stderr)
            push(0)
            pc += 1

        else:
            dump_state(f"Unknown opcode {op}")
            return None

        step += 1

    dump_state("Fell off end of bytecode")
    return None


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------
def compile_stage_bc(base_dir, stage_num, timeout_sec=300):
    """Compile bytecodes for a pipeline stage using iris-stage0."""
    import subprocess
    cmd = [
        os.path.join(base_dir, "bootstrap/iris-stage0"), "run",
        os.path.join(base_dir, "src/iris-programs/compiler/save_pipeline_bc.iris"),
        str(stage_num), "0"
    ]
    print(f"Running: {' '.join(cmd)}", file=sys.stderr)
    result = subprocess.run(
        cmd, capture_output=True, text=True, timeout=timeout_sec, cwd=base_dir
    )
    if result.stderr:
        print(f"  stderr: {result.stderr[:500]}", file=sys.stderr)
    print(f"  stdout length: {len(result.stdout)}", file=sys.stderr)
    print(f"  return code: {result.returncode}", file=sys.stderr)
    if result.returncode != 0:
        raise RuntimeError(f"Stage {stage_num} compilation failed (rc={result.returncode})")
    return parse_bytecodes(result.stdout)


def run_stage(label, bc, input_val, extra_inputs=None, max_steps=2_000_000,
              trace_all=False, trace_file=None):
    """Run a pipeline stage and report results."""
    if trace_file is None:
        trace_file = sys.stderr

    print(f"\n{'='*60}", file=trace_file)
    print(f"Running {label} ({len(bc)} bytecodes)", file=trace_file)
    print(f"  Primary input: {val_repr(input_val, max_depth=2, max_items=8)}", file=trace_file)
    if extra_inputs:
        for slot, val in extra_inputs.items():
            print(f"  Extra input [slot {slot}]: {val_repr(val, max_depth=1, max_items=5)}", file=trace_file)
    print(f"  First 40 bytecodes: {bc[:40]}", file=trace_file)
    print(f"{'='*60}", file=trace_file)

    result = run_vm(bc, input_val, max_steps=max_steps, trace_all=trace_all,
                    trace_file=trace_file, extra_inputs=extra_inputs)

    if result is not None:
        print(f"\n=== {label}: completed successfully ===", file=trace_file)
        print(f"Result: {val_repr(result, max_depth=3, max_items=20)}", file=trace_file)
    else:
        print(f"\n=== {label}: FAILED ===", file=trace_file)

    return result


def main():
    import subprocess
    import argparse

    parser = argparse.ArgumentParser(description="IRIS bytecode interpreter for debugging")
    parser.add_argument("--stage", type=int, default=None,
                        help="Pipeline stage: 0=tokenizer, 1=parser, 2=AST compiler")
    parser.add_argument("--trace", action="store_true",
                        help="Print every opcode execution")
    parser.add_argument("--max-steps", type=int, default=2_000_000,
                        help="Max steps before aborting (default 2M)")
    parser.add_argument("--input", type=str, default=None,
                        help="Source code to process (default: 'let f n = n + 1\\n')")
    parser.add_argument("bc_file", nargs="?", default=None,
                        help="Bytecode file to read (stage 0 mode only)")
    args = parser.parse_args()

    base_dir = os.path.dirname(os.path.abspath(__file__)) or "."
    input_str = args.input if args.input is not None else "let f n = n + 1\n"

    # Default: if no --stage and no bc_file, stage 0
    if args.stage is None and args.bc_file is None:
        args.stage = 0
    elif args.stage is None:
        args.stage = 0  # bc_file implies stage 0

    # -----------------------------------------------------------------------
    # Stage 0: Tokenizer
    # -----------------------------------------------------------------------
    if args.bc_file:
        print(f"Reading bytecodes from file: {args.bc_file}", file=sys.stderr)
        with open(args.bc_file, "r") as f:
            raw = f.read()
        tok_bc = parse_bytecodes(raw)
    elif args.stage >= 0:
        tok_bc = compile_stage_bc(base_dir, 0, timeout_sec=120)
    else:
        tok_bc = None

    if args.stage == 0:
        result = run_stage("Stage 0 (tokenizer)", tok_bc, input_str,
                           max_steps=args.max_steps, trace_all=args.trace)
        if result is None:
            sys.exit(1)
        sys.exit(0)

    # Run tokenizer to get tokens for parser input
    print(f"\n--- Running tokenizer to produce parser input ---", file=sys.stderr)
    tokens = run_stage("Stage 0 (tokenizer → parser input)", tok_bc, input_str,
                       max_steps=args.max_steps, trace_all=False)
    if tokens is None:
        print("FATAL: Tokenizer failed, cannot proceed to parser", file=sys.stderr)
        sys.exit(1)

    print(f"\nTokenizer produced {tuple_len(tokens)} tokens:", file=sys.stderr)
    if is_tuple(tokens):
        for i, tok in enumerate(tokens):
            print(f"  token[{i}]: {val_repr(tok, max_depth=2, max_items=10)}", file=sys.stderr)

    # -----------------------------------------------------------------------
    # Stage 1: Parser
    # -----------------------------------------------------------------------
    if args.stage >= 1:
        par_bc = compile_stage_bc(base_dir, 1, timeout_sec=600)

        # Parser takes 2 args: tokens (slot 0, pushed on stack) and source (slot 1)
        # Native VM: rsi→stack+locals[0], rdx→locals[1]
        result = run_stage("Stage 1 (parser)", par_bc, tokens,
                           extra_inputs={1: input_str},
                           max_steps=args.max_steps, trace_all=args.trace)
        if result is None:
            print("FATAL: Parser failed", file=sys.stderr)
            sys.exit(1)

        if args.stage == 1:
            sys.exit(0)

        tree = result

    # -----------------------------------------------------------------------
    # Stage 2: AST compiler
    # -----------------------------------------------------------------------
    if args.stage >= 2:
        ast_bc = compile_stage_bc(base_dir, 2, timeout_sec=120)

        # AST compiler input: (list_append () tree) = 1-element tuple wrapping the tree
        ast_input = (tree,)
        result = run_stage("Stage 2 (AST compiler)", ast_bc, ast_input,
                           max_steps=args.max_steps, trace_all=args.trace)
        if result is None:
            print("FATAL: AST compiler failed", file=sys.stderr)
            sys.exit(1)

        sys.exit(0)


if __name__ == "__main__":
    main()
