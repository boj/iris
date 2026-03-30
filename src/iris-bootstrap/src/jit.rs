//! jit: Tagged-value JIT runtime for IRIS.
//!
//! Packs Value into i64 for JIT-compiled code. The JIT operates on i64
//! exclusively. Complex values are heap-boxed and accessed via pointer.
//!
//! Tagging scheme:
//!   Bit 0 = 0: immediate integer (value >> 1 to extract)
//!   Bit 0 = 1: heap pointer to Box<Value> (clear bit 0 for address)
//!
//! This lets the JIT handle ALL IRIS values as i64, including strings,
//! tuples, Programs, etc. Arithmetic on tagged ints is fast (just shift).
//! Complex operations call into Rust helpers that unbox/rebox.

use std::rc::Rc;
use iris_types::eval::Value;

/// Pack a Value into a tagged i64 for JIT code.
pub fn pack(v: Value) -> i64 {
    match v {
        Value::Int(n) => n << 1, // tag bit 0 = 0
        Value::Bool(b) => (b as i64) << 1,
        Value::Unit => 0, // same as Int(0) — Unit is falsy
        _ => {
            // Heap-box the value, return tagged pointer
            let boxed = Box::new(v);
            let ptr = Box::into_raw(boxed) as i64;
            ptr | 1 // tag bit 0 = 1
        }
    }
}

/// Unpack a tagged i64 back to Value.
pub fn unpack(tagged: i64) -> Value {
    if tagged & 1 == 0 {
        // Immediate integer
        Value::Int(tagged >> 1)
    } else {
        // Heap pointer — clone the boxed Value (don't consume the Box)
        let ptr = (tagged & !1) as *mut Value;
        if ptr.is_null() { return Value::Unit; }
        unsafe { (*ptr).clone() }
    }
}

/// Free a tagged heap value. Must be called when the JIT is done with a value.
pub fn free_tagged(tagged: i64) {
    if tagged & 1 == 1 {
        let ptr = (tagged & !1) as *mut Value;
        if !ptr.is_null() {
            unsafe { let _ = Box::from_raw(ptr); }
        }
    }
}

/// Check if a tagged value is an immediate integer.
pub fn is_int(tagged: i64) -> bool {
    tagged & 1 == 0
}

/// Extract immediate integer (assumes is_int is true).
pub fn get_int(tagged: i64) -> i64 {
    tagged >> 1
}

/// Make a tagged integer.
pub fn make_int(n: i64) -> i64 {
    n << 1
}

// ---------------------------------------------------------------------------
// JIT runtime helpers — called from JIT-generated code via function pointers
// ---------------------------------------------------------------------------

/// Prim dispatch: evaluate a primitive opcode on tagged arguments.
/// Called from JIT code for opcodes the JIT can't handle inline.
pub extern "C" fn rt_prim_dispatch(opcode: i64, a: i64, b: i64, c: i64) -> i64 {
    let va = unpack(a);
    let vb = unpack(b);
    let _vc = unpack(c);

    let result = match opcode as u8 {
        // Arithmetic (inline in JIT, but fallback here)
        0x00 => match (&va, &vb) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x + y),
            _ => Value::Unit,
        },
        0x01 => match (&va, &vb) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x - y),
            _ => Value::Unit,
        },
        0x02 => match (&va, &vb) {
            (Value::Int(x), Value::Int(y)) => Value::Int(x * y),
            _ => Value::Unit,
        },
        0x03 => match (&va, &vb) {
            (Value::Int(x), Value::Int(y)) if *y != 0 => Value::Int(x / y),
            _ => Value::Int(0),
        },
        // Comparisons
        0x20 => Value::Bool(va == vb),
        0x21 => Value::Bool(va != vb),
        0x22 => match (&va, &vb) { (Value::Int(x), Value::Int(y)) => Value::Bool(x < y), _ => Value::Bool(false) },
        0x23 => match (&va, &vb) { (Value::Int(x), Value::Int(y)) => Value::Bool(x > y), _ => Value::Bool(false) },
        0x24 => match (&va, &vb) { (Value::Int(x), Value::Int(y)) => Value::Bool(x <= y), _ => Value::Bool(false) },
        0x25 => match (&va, &vb) { (Value::Int(x), Value::Int(y)) => Value::Bool(x >= y), _ => Value::Bool(false) },

        // String ops
        0xB0 => match &va { Value::String(s) => Value::Int(s.len() as i64), _ => Value::Int(0) },
        0xC0 => match (&va, &vb) { // char_at
            (Value::String(s), Value::Int(i)) => {
                Value::Int(s.as_bytes().get(*i as usize).map(|&b| b as i64).unwrap_or(-1))
            }
            _ => Value::Int(-1),
        },
        0xB1 => match (&va, &vb) { // str_concat
            (Value::String(x), Value::String(y)) => Value::String(format!("{}{}", x, y)),
            _ => Value::Unit,
        },

        // Collection ops
        0xC1 => { // list_append
            let mut elems: Vec<Value> = match &va {
                Value::Tuple(t) => t.as_ref().clone(),
                Value::Unit => vec![],
                Value::Range(s, e) => if *e > *s { (*s..*e).map(Value::Int).collect() } else { vec![] },
                _ => vec![],
            };
            elems.push(vb);
            Value::tuple(elems)
        }
        0xC2 => match (&va, &vb) { // list_nth
            (Value::Tuple(t), Value::Int(i)) => t.get(*i as usize).cloned().unwrap_or(Value::Unit),
            (Value::Range(s, e), Value::Int(i)) => if s + i < *e { Value::Int(s + i) } else { Value::Unit },
            _ => Value::Unit,
        },
        0xC7 => match (&va, &vb) { // list_range
            (Value::Int(s), Value::Int(e)) => if *e <= *s { Value::Range(0, 0) } else { Value::Range(*s, *e) },
            _ => Value::Unit,
        },
        0xD2 => match (&va, &vb) { // tuple_get
            (Value::Tuple(t), Value::Int(i)) => t.get(*i as usize).cloned().unwrap_or(Value::Unit),
            _ => Value::Unit,
        },
        0xD6 => match &va { // tuple_len
            Value::Tuple(t) => Value::Int(t.len() as i64),
            Value::Unit => Value::Int(0),
            _ => Value::Int(0),
        },
        0xF0 => match &va { // list_len
            Value::Tuple(t) => Value::Int(t.len() as i64),
            Value::Range(s, e) => Value::Int(if *e > *s { *e - *s } else { 0 }),
            Value::String(s) => Value::Int(s.len() as i64),
            Value::Unit => Value::Int(0),
            _ => Value::Int(0),
        },

        // Graph introspection
        0x80 => Value::Unit, // self_graph — needs context, placeholder
        0x82 => match (&va, &vb) { // graph_get_kind
            (Value::Program(g), Value::Int(nid)) => {
                g.nodes.get(&iris_types::graph::NodeId(*nid as u64))
                    .map(|n| Value::Int(n.kind as i64))
                    .unwrap_or(Value::Int(-1))
            }
            _ => Value::Int(-1),
        },
        0x83 => match (&va, &vb) { // graph_get_prim_op
            (Value::Program(g), Value::Int(nid)) => {
                g.nodes.get(&iris_types::graph::NodeId(*nid as u64))
                    .and_then(|n| match &n.payload {
                        iris_types::graph::NodePayload::Prim { opcode } => Some(Value::Int(*opcode as i64)),
                        _ => None,
                    })
                    .unwrap_or(Value::Int(-1))
            }
            _ => Value::Int(-1),
        },
        0x8A => match &va { // graph_get_root
            Value::Program(g) => Value::Int(g.root.0 as i64),
            _ => Value::Int(-1),
        },
        0x8F => match (&va, &vb) { // graph_outgoing
            (Value::Program(g), Value::Int(nid)) => {
                let node_id = iris_types::graph::NodeId(*nid as u64);
                let children: Vec<Value> = g.edges.iter()
                    .filter(|e| e.source == node_id && e.label == iris_types::graph::EdgeLabel::Argument)
                    .map(|e| Value::Int(e.target.0 as i64))
                    .collect();
                Value::tuple(children)
            }
            _ => Value::Unit,
        },
        0xEE => match (&va, &vb) { // graph_set_root
            (Value::Program(g), Value::Int(nid)) => {
                let mut new_g = g.as_ref().clone();
                new_g.root = iris_types::graph::NodeId(*nid as u64);
                Value::Program(Rc::new(new_g))
            }
            _ => Value::Unit,
        },
        0x89 => { // graph_eval — evaluate a sub-program
            // This is the critical recursive call. We evaluate the target
            // program through mini_eval (for now). The JIT handles the
            // outer loop natively; graph_eval is the slow path.
            match &va {
                Value::Program(g) => {
                    let inputs: Vec<Value> = match &vb {
                        Value::Tuple(elems) => elems.as_ref().clone(),
                        other => vec![other.clone()],
                    };
                    let empty_reg = std::collections::BTreeMap::new();
                    crate::mini_eval::evaluate_with_registry(g, &inputs, 10_000_000, &empty_reg)
                        .unwrap_or(Value::Unit)
                }
                _ => Value::Unit,
            }
        },

        // Default: return Unit
        _ => Value::Unit,
    };

    pack(result)
}

/// Tuple construction: pack multiple tagged values into a tagged tuple.
pub extern "C" fn rt_make_tuple(count: i64, values_ptr: *const i64) -> i64 {
    let values: Vec<Value> = (0..count as usize).map(|i| {
        let tagged = unsafe { *values_ptr.add(i) };
        unpack(tagged)
    }).collect();
    pack(Value::tuple(values))
}

/// Graph eval for self-interpreter: fn(me, program, env) -> result
/// This is the trampoline for recursive self-interpreter calls.
pub extern "C" fn rt_self_eval(me: i64, program: i64, env: i64) -> i64 {
    let me_val = unpack(me);
    let prog_val = unpack(program);
    let env_val = unpack(env);

    match &me_val {
        Value::Program(interp) => {
            let empty_reg = std::collections::BTreeMap::new();
            let result = crate::mini_eval::evaluate_with_registry(
                interp,
                &[me_val.clone(), prog_val, env_val],
                10_000_000,
                &empty_reg,
            ).unwrap_or(Value::Unit);
            pack(result)
        }
        _ => 0, // Unit
    }
}
