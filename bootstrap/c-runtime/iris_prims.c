/*
 * iris_prims.c -- Value constructors and primitive dispatch.
 *
 * This file provides:
 *   1. iris_value_t constructors (iris_int, iris_float64, etc.)
 *   2. Value-level graph introspection wrappers
 *   3. Arithmetic / comparison / list primitive dispatch
 */

#include "iris_rt.h"
#include "iris_graph.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <math.h>

/* -----------------------------------------------------------------------
 * Simple arena: bump-allocate values from a pool.
 * This avoids per-value malloc overhead. The arena is never freed during
 * a single evaluation run (matches Rc<> semantics in the Rust evaluator).
 * ----------------------------------------------------------------------- */

#define ARENA_BLOCK_SIZE (1 << 16)  /* 64 KiB */

typedef struct arena_block {
    struct arena_block *next;
    size_t              used;
    char                data[ARENA_BLOCK_SIZE];
} arena_block_t;

static arena_block_t *arena_head = NULL;

static void *arena_alloc(size_t size) {
    /* Align to 8 bytes */
    size = (size + 7) & ~(size_t)7;
    if (!arena_head || arena_head->used + size > ARENA_BLOCK_SIZE) {
        arena_block_t *blk = malloc(sizeof(arena_block_t));
        if (!blk) { fprintf(stderr, "out of memory\n"); abort(); }
        blk->next  = arena_head;
        blk->used  = 0;
        arena_head = blk;
    }
    void *ptr = &arena_head->data[arena_head->used];
    arena_head->used += size;
    return ptr;
}

/* -----------------------------------------------------------------------
 * Value constructors
 * ----------------------------------------------------------------------- */

iris_value_t *iris_int(int64_t n) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag = IRIS_INT;
    v->i   = n;
    return v;
}

iris_value_t *iris_float64(double f) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag = IRIS_FLOAT64;
    v->f   = f;
    return v;
}

iris_value_t *iris_bool(bool b) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag = IRIS_BOOL;
    v->b   = b;
    return v;
}

static iris_value_t unit_singleton = { .tag = IRIS_UNIT };

iris_value_t *iris_unit(void) {
    return &unit_singleton;
}

iris_value_t *iris_tuple(iris_value_t **elems, uint32_t len) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag = IRIS_TUPLE;
    v->tuple.len = len;
    if (len > 0) {
        v->tuple.elems = arena_alloc(len * sizeof(iris_value_t *));
        memcpy(v->tuple.elems, elems, len * sizeof(iris_value_t *));
    } else {
        v->tuple.elems = NULL;
    }
    return v;
}

iris_value_t *iris_program(iris_graph_t *g) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag   = IRIS_PROGRAM;
    v->graph = g;
    return v;
}

iris_value_t *iris_tagged(uint16_t tag, iris_value_t *payload) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag             = IRIS_TAGGED;
    v->tagged.tag_index = tag;
    v->tagged.payload   = payload;
    return v;
}

iris_value_t *iris_string(const char *data, uint32_t len) {
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag      = IRIS_STRING;
    v->str.data = arena_alloc(len + 1);
    memcpy(v->str.data, data, len);
    v->str.data[len] = '\0';
    v->str.len  = len;
    return v;
}

/* -----------------------------------------------------------------------
 * Value-level graph introspection wrappers
 * ----------------------------------------------------------------------- */

iris_value_t *iris_graph_get_root(iris_value_t *prog) {
    if (!prog || prog->tag != IRIS_PROGRAM || !prog->graph) return iris_int(-1);
    return iris_int((int64_t)prog->graph->root);
}

iris_value_t *iris_graph_get_kind(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node) return iris_int(-1);
    return iris_int((int64_t)node->kind);
}

iris_value_t *iris_graph_get_prim_op(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_PRIM) return iris_int(-1);
    return iris_int((int64_t)node->payload.prim_opcode);
}

iris_value_t *iris_graph_outgoing(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_tuple(NULL, 0);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);

    uint64_t targets[256];
    uint32_t count = iris_graph_raw_outgoing(g, nid, EL_ARGUMENT, targets, 256);

    iris_value_t **elems = arena_alloc(count * sizeof(iris_value_t *));
    for (uint32_t i = 0; i < count; i++) {
        elems[i] = iris_int((int64_t)targets[i]);
    }
    return iris_tuple(elems, count);
}

iris_value_t *iris_graph_set_root(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_unit();
    iris_graph_t *cloned = iris_graph_raw_clone(prog->graph);
    cloned->root = (uint64_t)iris_as_int(node_id);
    return iris_program(cloned);
}

iris_value_t *iris_graph_eval(iris_value_t *prog, iris_value_t *inputs) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_unit();
    return iris_eval_graph(prog->graph, inputs);
}

iris_value_t *iris_graph_eval_env(iris_value_t *prog, iris_value_t *binder,
                                  iris_value_t *value, iris_value_t *inputs) {
    (void)binder;
    (void)value;
    (void)inputs;
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_unit();
    /* Simplified: evaluate the graph with the provided input.
     * Full env binding would set binder=value in the environment. */
    return iris_eval_graph(prog->graph, inputs ? inputs : iris_unit());
}

iris_value_t *iris_graph_edge_target(iris_value_t *prog, iris_value_t *src,
                                     iris_value_t *port, iris_value_t *label) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t source = (uint64_t)iris_as_int(src);
    uint8_t  p      = (uint8_t)iris_as_int(port);
    uint8_t  l      = (uint8_t)iris_as_int(label);
    uint64_t target;
    if (iris_graph_raw_edge_target(g, source, p, l, &target)) {
        return iris_int((int64_t)target);
    }
    return iris_int(-1);
}

iris_value_t *iris_graph_get_binder(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node) return iris_int(-1);
    switch (node->kind) {
        case NK_LAMBDA:  return iris_int((int64_t)node->payload.lambda.binder_id);
        case NK_LET_REC: return iris_int((int64_t)node->payload.letrec.binder_id);
        default:         return iris_int(-1);
    }
}

iris_value_t *iris_graph_get_tag(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_INJECT) return iris_int(-1);
    return iris_int((int64_t)node->payload.tag_index);
}

iris_value_t *iris_graph_get_field_index(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_PROJECT) return iris_int(-1);
    return iris_int((int64_t)node->payload.field_index);
}

iris_value_t *iris_graph_get_effect_tag(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_EFFECT) return iris_int(-1);
    return iris_int((int64_t)node->payload.effect_tag);
}

iris_value_t *iris_graph_get_lit_type_tag(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_int(-1);
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_LIT) return iris_int(-1);
    return iris_int((int64_t)node->payload.lit.type_tag);
}

iris_value_t *iris_graph_get_lit_value(iris_value_t *prog, iris_value_t *node_id) {
    if (!prog || prog->tag != IRIS_PROGRAM) return iris_unit();
    iris_graph_t *g = prog->graph;
    uint64_t nid = (uint64_t)iris_as_int(node_id);
    iris_node_t *node = iris_graph_raw_find_node(g, nid);
    if (!node || node->kind != NK_LIT) return iris_unit();

    iris_lit_payload_t *lit = &node->payload.lit;
    switch (lit->type_tag) {
        case 0x00: /* Int */
            if (lit->value_len >= 8) {
                int64_t val;
                memcpy(&val, lit->value, 8);
                return iris_int(val);
            }
            return iris_int(0);
        case 0x02: /* Float64 */
            if (lit->value_len >= 8) {
                double val;
                memcpy(&val, lit->value, 8);
                return iris_float64(val);
            }
            return iris_float64(0.0);
        case 0x04: /* Bool */
            if (lit->value_len >= 1) {
                return iris_bool(lit->value[0] != 0);
            }
            return iris_bool(false);
        case 0x06: /* Unit */
            return iris_unit();
        case 0xFF: /* InputRef */
            /* Return the input index as an integer */
            if (lit->value_len >= 1) {
                return iris_int((int64_t)lit->value[0]);
            }
            return iris_int(0);
        default:
            return iris_unit();
    }
}

iris_value_t *iris_value_get_tag(iris_value_t *v) {
    if (!v || v->tag != IRIS_TAGGED) return iris_int(-1);
    return iris_int((int64_t)v->tagged.tag_index);
}

iris_value_t *iris_value_get_payload(iris_value_t *v) {
    if (!v || v->tag != IRIS_TAGGED) return iris_unit();
    return v->tagged.payload;
}

iris_value_t *iris_value_make_tagged(iris_value_t *tag, iris_value_t *payload) {
    uint16_t t = (uint16_t)iris_as_int(tag);
    return iris_tagged(t, payload);
}

iris_value_t *iris_perform_effect(iris_value_t *tag, iris_value_t *args) {
    int64_t t = iris_as_int(tag);
    /* For now, print effects go to stderr */
    if (t == 0xF4) { /* print */
        if (args && args->tag == IRIS_STRING) {
            fprintf(stdout, "%s", args->str.data);
        } else if (args && args->tag == IRIS_INT) {
            fprintf(stdout, "%ld", (long)args->i);
        } else {
            fprintf(stdout, "<value>");
        }
        return iris_unit();
    }
    fprintf(stderr, "warning: unhandled effect tag 0x%lx\n", (long)t);
    return iris_unit();
}

iris_value_t *iris_list_len(iris_value_t *v) {
    if (!v) return iris_int(0);
    if (v->tag == IRIS_TUPLE)  return iris_int((int64_t)v->tuple.len);
    if (v->tag == IRIS_UNIT)   return iris_int(0);
    return iris_int(0);
}

iris_value_t *iris_list_nth(iris_value_t *list, iris_value_t *idx) {
    if (!list || list->tag != IRIS_TUPLE) return iris_unit();
    int64_t i = iris_as_int(idx);
    if (i < 0 || (uint32_t)i >= list->tuple.len) return iris_unit();
    return list->tuple.elems[i];
}

/* -----------------------------------------------------------------------
 * Arithmetic helpers
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_add(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_float64(iris_as_float(args[0]) + iris_as_float(args[1]));
    return iris_int(iris_as_int(args[0]) + iris_as_int(args[1]));
}

static iris_value_t *prim_sub(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_float64(iris_as_float(args[0]) - iris_as_float(args[1]));
    return iris_int(iris_as_int(args[0]) - iris_as_int(args[1]));
}

static iris_value_t *prim_mul(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_float64(iris_as_float(args[0]) * iris_as_float(args[1]));
    return iris_int(iris_as_int(args[0]) * iris_as_int(args[1]));
}

static iris_value_t *prim_div(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64) {
        double b = iris_as_float(args[1]);
        if (b == 0.0) return iris_float64(0.0);
        return iris_float64(iris_as_float(args[0]) / b);
    }
    int64_t b = iris_as_int(args[1]);
    if (b == 0) return iris_int(0);
    return iris_int(iris_as_int(args[0]) / b);
}

static iris_value_t *prim_mod(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    int64_t b = iris_as_int(args[1]);
    if (b == 0) return iris_int(0);
    return iris_int(iris_as_int(args[0]) % b);
}

static iris_value_t *prim_neg(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_int(0);
    if (args[0]->tag == IRIS_FLOAT64)
        return iris_float64(-args[0]->f);
    return iris_int(-iris_as_int(args[0]));
}

static iris_value_t *prim_eq(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(false);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) == iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) == iris_as_int(args[1]));
}

static iris_value_t *prim_neq(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(true);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) != iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) != iris_as_int(args[1]));
}

static iris_value_t *prim_lt(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(false);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) < iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) < iris_as_int(args[1]));
}

static iris_value_t *prim_gt(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(false);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) > iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) > iris_as_int(args[1]));
}

static iris_value_t *prim_le(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(true);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) <= iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) <= iris_as_int(args[1]));
}

static iris_value_t *prim_ge(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_bool(true);
    if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
        return iris_bool(iris_as_float(args[0]) >= iris_as_float(args[1]));
    return iris_bool(iris_as_int(args[0]) >= iris_as_int(args[1]));
}

static iris_value_t *prim_int_to_float(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_float64(0.0);
    return iris_float64((double)iris_as_int(args[0]));
}

static iris_value_t *prim_float_to_int(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_int(0);
    return iris_int((int64_t)iris_as_float(args[0]));
}

static iris_value_t *prim_bool_to_int(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_int(0);
    if (args[0]->tag == IRIS_BOOL) return iris_int(args[0]->b ? 1 : 0);
    if (args[0]->tag == IRIS_INT) return args[0];
    return iris_int(0);
}

/* Bitwise */
static iris_value_t *prim_bitand(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    return iris_int(iris_as_int(args[0]) & iris_as_int(args[1]));
}

static iris_value_t *prim_bitor(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    return iris_int(iris_as_int(args[0]) | iris_as_int(args[1]));
}

static iris_value_t *prim_bitxor(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    return iris_int(iris_as_int(args[0]) ^ iris_as_int(args[1]));
}

static iris_value_t *prim_bitnot(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_int(0);
    return iris_int(~iris_as_int(args[0]));
}

static iris_value_t *prim_shl(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    return iris_int(iris_as_int(args[0]) << iris_as_int(args[1]));
}

static iris_value_t *prim_shr(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_int(0);
    return iris_int(iris_as_int(args[0]) >> iris_as_int(args[1]));
}

/* List operations */
static iris_value_t *prim_list_concat_impl(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_tuple(NULL, 0);
    iris_value_t *a = args[0];
    iris_value_t *b = args[1];
    uint32_t la = (a && a->tag == IRIS_TUPLE) ? a->tuple.len : 0;
    uint32_t lb = (b && b->tag == IRIS_TUPLE) ? b->tuple.len : 0;
    uint32_t total = la + lb;
    iris_value_t **elems = arena_alloc(total * sizeof(iris_value_t *));
    if (a && a->tag == IRIS_TUPLE)
        memcpy(elems, a->tuple.elems, la * sizeof(iris_value_t *));
    if (b && b->tag == IRIS_TUPLE)
        memcpy(elems + la, b->tuple.elems, lb * sizeof(iris_value_t *));
    return iris_tuple(elems, total);
}

static iris_value_t *prim_list_append(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_tuple(NULL, 0);
    iris_value_t *lst = args[0];
    iris_value_t *elem = args[1];
    uint32_t la = (lst && lst->tag == IRIS_TUPLE) ? lst->tuple.len : 0;
    uint32_t total = la + 1;
    iris_value_t **elems = arena_alloc(total * sizeof(iris_value_t *));
    if (lst && lst->tag == IRIS_TUPLE)
        memcpy(elems, lst->tuple.elems, la * sizeof(iris_value_t *));
    elems[la] = elem;
    return iris_tuple(elems, total);
}

/* Math */
static iris_value_t *prim_math_sqrt(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_float64(0.0);
    return iris_float64(sqrt(iris_as_float(args[0])));
}

static iris_value_t *prim_math_log(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_float64(0.0);
    return iris_float64(log(iris_as_float(args[0])));
}

static iris_value_t *prim_math_exp(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_float64(0.0);
    return iris_float64(exp(iris_as_float(args[0])));
}

/* String operations */
static iris_value_t *prim_str_len(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1 || !args[0] || args[0]->tag != IRIS_STRING) return iris_int(0);
    return iris_int((int64_t)args[0]->str.len);
}

static iris_value_t *prim_str_concat(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_string("", 0);
    iris_value_t *a = args[0];
    iris_value_t *b = args[1];
    uint32_t la = (a && a->tag == IRIS_STRING) ? a->str.len : 0;
    uint32_t lb = (b && b->tag == IRIS_STRING) ? b->str.len : 0;
    uint32_t total = la + lb;
    char *buf = arena_alloc(total + 1);
    if (la > 0) memcpy(buf, a->str.data, la);
    if (lb > 0) memcpy(buf + la, b->str.data, lb);
    buf[total] = '\0';
    iris_value_t *v = arena_alloc(sizeof(iris_value_t));
    v->tag      = IRIS_STRING;
    v->str.data = buf;
    v->str.len  = total;
    return v;
}

static iris_value_t *prim_int_to_string(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_string("0", 1);
    char buf[32];
    int len = snprintf(buf, sizeof(buf), "%ld", (long)iris_as_int(args[0]));
    return iris_string(buf, (uint32_t)len);
}

static iris_value_t *prim_str_to_int(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1 || !args[0] || args[0]->tag != IRIS_STRING) return iris_int(0);
    return iris_int(atol(args[0]->str.data));
}

static iris_value_t *prim_tuple_get(iris_value_t **args, uint32_t nargs) {
    if (nargs < 2) return iris_unit();
    return iris_list_nth(args[0], args[1]);
}

static iris_value_t *prim_tuple_len(iris_value_t **args, uint32_t nargs) {
    if (nargs < 1) return iris_int(0);
    return iris_list_len(args[0]);
}

static iris_value_t *prim_print(iris_value_t **args, uint32_t nargs) {
    for (uint32_t i = 0; i < nargs; i++) {
        iris_value_t *v = args[i];
        if (!v) continue;
        switch (v->tag) {
            case IRIS_INT:     fprintf(stdout, "%ld", (long)v->i); break;
            case IRIS_FLOAT64: fprintf(stdout, "%g", v->f); break;
            case IRIS_BOOL:    fprintf(stdout, "%s", v->b ? "true" : "false"); break;
            case IRIS_UNIT:    fprintf(stdout, "()"); break;
            case IRIS_STRING:  fprintf(stdout, "%s", v->str.data); break;
            default:           fprintf(stdout, "<value:%d>", v->tag); break;
        }
    }
    fprintf(stdout, "\n");
    fflush(stdout);
    return iris_unit();
}

/* -----------------------------------------------------------------------
 * Prim dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_prim(uint8_t opcode, iris_value_t **args, uint32_t nargs,
                             iris_value_t *self_program) {
    switch (opcode) {
        /* Arithmetic */
        case 0x00: return prim_add(args, nargs);
        case 0x01: return prim_sub(args, nargs);
        case 0x02: return prim_mul(args, nargs);
        case 0x03: return prim_div(args, nargs);
        case 0x04: return prim_mod(args, nargs);
        case 0x05: return prim_neg(args, nargs);
        case 0x06: /* abs */
            if (nargs < 1) return iris_int(0);
            if (args[0]->tag == IRIS_FLOAT64) return iris_float64(fabs(args[0]->f));
            { int64_t x = iris_as_int(args[0]); return iris_int(x < 0 ? -x : x); }
        case 0x07: /* min */
            if (nargs < 2) return iris_int(0);
            if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
                return iris_float64(fmin(iris_as_float(args[0]), iris_as_float(args[1])));
            { int64_t a = iris_as_int(args[0]), b = iris_as_int(args[1]);
              return iris_int(a < b ? a : b); }
        case 0x08: /* max */
            if (nargs < 2) return iris_int(0);
            if (args[0]->tag == IRIS_FLOAT64 || args[1]->tag == IRIS_FLOAT64)
                return iris_float64(fmax(iris_as_float(args[0]), iris_as_float(args[1])));
            { int64_t a = iris_as_int(args[0]), b = iris_as_int(args[1]);
              return iris_int(a > b ? a : b); }

        /* Comparison */
        case 0x20: return prim_eq(args, nargs);
        case 0x21: return prim_neq(args, nargs);
        case 0x22: return prim_lt(args, nargs);
        case 0x23: return prim_gt(args, nargs);
        case 0x24: return prim_le(args, nargs);
        case 0x25: return prim_ge(args, nargs);

        /* Conversions */
        case 0x40: return prim_int_to_float(args, nargs);
        case 0x41: return prim_float_to_int(args, nargs);
        case 0x44: return prim_bool_to_int(args, nargs);

        /* Bitwise */
        case 0x10: return prim_bitand(args, nargs);
        case 0x11: return prim_bitor(args, nargs);
        case 0x12: return prim_bitxor(args, nargs);
        case 0x13: return prim_bitnot(args, nargs);
        case 0x14: return prim_shl(args, nargs);
        case 0x15: return prim_shr(args, nargs);

        /* Graph self-reflection: 0x80 = self_program */
        case 0x80: return self_program ? self_program : iris_unit();

        /* Graph introspection */
        case 0x82: /* graph_get_kind */
            return (nargs >= 2) ? iris_graph_get_kind(args[0], args[1]) : iris_int(-1);
        case 0x83: /* graph_get_prim_op */
            return (nargs >= 2) ? iris_graph_get_prim_op(args[0], args[1]) : iris_int(-1);
        case 0x89: /* graph_eval */
            return (nargs >= 1) ? iris_graph_eval(args[0], nargs >= 2 ? args[1] : iris_unit()) : iris_unit();
        case 0x8A: /* graph_get_root */
            return (nargs >= 1) ? iris_graph_get_root(args[0]) : iris_int(-1);
        case 0x8F: /* graph_outgoing */
            return (nargs >= 2) ? iris_graph_outgoing(args[0], args[1]) : iris_tuple(NULL, 0);

        /* More graph ops */
        case 0x66: /* graph_get_lit_type_tag */
            return (nargs >= 2) ? iris_graph_get_lit_type_tag(args[0], args[1]) : iris_int(-1);
        case 0x8E: /* graph_get_lit_value */
            return (nargs >= 2) ? iris_graph_get_lit_value(args[0], args[1]) : iris_unit();
        case 0x97: /* graph_edge_target */
            return (nargs >= 4) ? iris_graph_edge_target(args[0], args[1], args[2], args[3]) : iris_int(-1);
        case 0x98: /* graph_get_binder */
            return (nargs >= 2) ? iris_graph_get_binder(args[0], args[1]) : iris_int(-1);
        case 0x99: /* graph_eval_env */
            return (nargs >= 3) ? iris_graph_eval_env(args[0], args[1], args[2],
                                                       nargs >= 4 ? args[3] : iris_unit()) : iris_unit();
        case 0x9A: /* graph_get_tag */
            return (nargs >= 2) ? iris_graph_get_tag(args[0], args[1]) : iris_int(-1);
        case 0x9B: /* graph_get_field_index */
            return (nargs >= 2) ? iris_graph_get_field_index(args[0], args[1]) : iris_int(-1);
        case 0x9C: /* value_get_tag */
            return (nargs >= 1) ? iris_value_get_tag(args[0]) : iris_int(-1);
        case 0x9D: /* value_get_payload */
            return (nargs >= 1) ? iris_value_get_payload(args[0]) : iris_unit();
        case 0x9E: /* value_make_tagged */
            return (nargs >= 2) ? iris_value_make_tagged(args[0], args[1]) : iris_unit();
        case 0x9F: /* graph_get_effect_tag */
            return (nargs >= 2) ? iris_graph_get_effect_tag(args[0], args[1]) : iris_int(-1);
        case 0xA1: /* perform_effect */
            return (nargs >= 2) ? iris_perform_effect(args[0], args[1]) : iris_unit();
        case 0xEE: /* graph_set_root */
            return (nargs >= 2) ? iris_graph_set_root(args[0], args[1]) : iris_unit();

        /* List / tuple operations */
        case 0x22 + 0x10: /* 0x32 = zip - simplified */
            return iris_tuple(NULL, 0);
        case 0x35: /* list_concat */
            return prim_list_concat_impl(args, nargs);
        case 0xC0: { /* char_at(string, index) → int (char code) */
            if (nargs < 2) return iris_int(0);
            if (args[0] && args[0]->tag == IRIS_STRING) {
                int64_t idx = iris_as_int(args[1]);
                if (idx >= 0 && (uint32_t)idx < args[0]->str.len) {
                    return iris_int((unsigned char)args[0]->str.data[idx]);
                }
            }
            return iris_int(0);
        }
        case 0xC1: /* list_append */
            return prim_list_append(args, nargs);
        case 0xC2: /* list_nth */
            return (nargs >= 2) ? iris_list_nth(args[0], args[1]) : iris_unit();
        case 0xD2: /* tuple_get */
            return prim_tuple_get(args, nargs);
        case 0xD6: /* tuple_len */
            return prim_tuple_len(args, nargs);
        case 0xC3: /* list_take */
            if (nargs < 2) return iris_tuple(NULL, 0);
            if (args[0] && args[0]->tag == IRIS_TUPLE) {
                int64_t n = iris_as_int(args[1]);
                uint32_t take = (n < 0) ? 0 : ((uint32_t)n > args[0]->tuple.len ? args[0]->tuple.len : (uint32_t)n);
                return iris_tuple(args[0]->tuple.elems, take);
            }
            return iris_tuple(NULL, 0);
        case 0xC4: /* list_drop */
            if (nargs < 2) return iris_tuple(NULL, 0);
            if (args[0] && args[0]->tag == IRIS_TUPLE) {
                int64_t n = iris_as_int(args[1]);
                uint32_t drop = (n < 0) ? 0 : (uint32_t)n;
                if (drop >= args[0]->tuple.len) return iris_tuple(NULL, 0);
                return iris_tuple(args[0]->tuple.elems + drop, args[0]->tuple.len - drop);
            }
            return iris_tuple(NULL, 0);
        case 0xC7: { /* list_range */
            if (nargs < 2) return iris_tuple(NULL, 0);
            int64_t lo = iris_as_int(args[0]);
            int64_t hi = iris_as_int(args[1]);
            if (hi <= lo) return iris_tuple(NULL, 0);
            uint32_t count = (uint32_t)(hi - lo);
            if (count > 10000) count = 10000;
            iris_value_t **elems = arena_alloc(count * sizeof(iris_value_t *));
            for (uint32_t i = 0; i < count; i++) {
                elems[i] = iris_int(lo + (int64_t)i);
            }
            return iris_tuple(elems, count);
        }
        case 0xCE: /* list_concat (alias) */
            return prim_list_concat_impl(args, nargs);
        case 0xF0: /* list_len */
            return (nargs >= 1) ? iris_list_len(args[0]) : iris_int(0);

        /* Graph construction & mutation */
        case 0xED: { /* graph_new */
            iris_graph_t *g = iris_graph_raw_alloc();

            return iris_program(g);
        }

        case 0x84: { /* graph_add_node_rt(program, kind) → node_id */
            if (nargs < 2) return iris_int(-1);
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            uint8_t nk = (uint8_t)iris_as_int(args[1]);
            /* Generate a unique node ID from node count + salt */
            uint64_t nid = (uint64_t)g->node_count + 1000;
            iris_node_t node = {0};
            node.id = nid;
            node.kind = nk;
            iris_graph_raw_add_node(g, node);
            return iris_int((int64_t)nid);
        }

        case 0x85: { /* graph_connect(program, src, tgt) → program */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            uint64_t src = (uint64_t)iris_as_int(args[1]);
            uint64_t tgt = (uint64_t)iris_as_int(args[2]);
            /* Find existing edge count from src to determine port */
            uint8_t port = 0;
            for (uint32_t i = 0; i < g->edge_count; i++) {
                if (g->edges[i].source == src && g->edges[i].label == EL_ARGUMENT) port++;
            }
            iris_edge_t e = { .source = src, .target = tgt, .port = port, .label = EL_ARGUMENT };
            iris_graph_raw_add_edge(g, e);
            return args[0];
        }

        case 0x86: { /* graph_set_prim_op(program, node_id, opcode) → program */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            iris_node_t *n = iris_graph_raw_find_node(g, (uint64_t)iris_as_int(args[1]));
            if (n) n->payload.prim_opcode = (uint8_t)iris_as_int(args[2]);
            return args[0];
        }

        case 0x87: { /* graph_disconnect(program, src, tgt) → program */
            /* Simplified: remove first edge from src to tgt */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            uint64_t src = (uint64_t)iris_as_int(args[1]);
            uint64_t tgt = (uint64_t)iris_as_int(args[2]);
            for (uint32_t i = 0; i < g->edge_count; i++) {
                if (g->edges[i].source == src && g->edges[i].target == tgt) {
                    g->edges[i] = g->edges[g->edge_count - 1];
                    g->edge_count--;
                    break;
                }
            }
            return args[0];
        }

        case 0x88: { /* graph_replace_subtree(program, old, new) → program */
            /* Replace all edges pointing to old_id with new_id */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            uint64_t old_id = (uint64_t)iris_as_int(args[1]);
            uint64_t new_id = (uint64_t)iris_as_int(args[2]);
            for (uint32_t i = 0; i < g->edge_count; i++) {
                if (g->edges[i].target == old_id) g->edges[i].target = new_id;
            }
            if (g->root == old_id) g->root = new_id;
            return args[0];
        }

        case 0x8B: { /* graph_add_guard_rt(program, pred, body, fallback) → node_id */
            if (nargs < 4) return iris_int(-1);
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return iris_int(-1);
            uint64_t nid = (uint64_t)g->node_count + 1000;
            iris_node_t node = {0};
            node.id = nid;
            node.kind = NK_GUARD;
            node.payload.guard.predicate_node = (uint64_t)iris_as_int(args[1]);
            node.payload.guard.body_node = (uint64_t)iris_as_int(args[2]);
            node.payload.guard.fallback_node = (uint64_t)iris_as_int(args[3]);
            iris_graph_raw_add_node(g, node);
            return iris_int((int64_t)nid);
        }

        case 0x8C: { /* graph_add_ref_rt(program, target_node) → node_id */
            if (nargs < 2) return iris_int(-1);
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return iris_int(-1);
            uint64_t nid = (uint64_t)g->node_count + 1000;
            iris_node_t node = {0};
            node.id = nid;
            node.kind = NK_REF;
            iris_graph_raw_add_node(g, node);
            iris_edge_t e = { .source = nid, .target = (uint64_t)iris_as_int(args[1]), .port = 0, .label = EL_ARGUMENT };
            iris_graph_raw_add_edge(g, e);
            return iris_int((int64_t)nid);
        }

        case 0x8D: { /* graph_set_cost(program, node_id, cost) → program */
            /* Cost is stored as node metadata — we don't track it in C */
            return (nargs >= 1) ? args[0] : iris_unit();
        }

        case 0x96: { /* graph_set_node_type(program, node_id, type_id) → program */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            iris_node_t *n = iris_graph_raw_find_node(g, (uint64_t)iris_as_int(args[1]));
            if (n) n->type_sig = (uint64_t)iris_as_int(args[2]);
            return args[0];
        }

        case 0xF1: { /* graph_set_lit_value(program, node_id, value) → program */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            iris_node_t *n = iris_graph_raw_find_node(g, (uint64_t)iris_as_int(args[1]));
            if (n && n->kind == NK_LIT) {
                int64_t val = iris_as_int(args[2]);
                if (!n->payload.lit.value) {
                    n->payload.lit.value = arena_alloc(8);
                    n->payload.lit.value_len = 8;
                }
                memcpy(n->payload.lit.value, &val, 8);
            }
            return args[0];
        }

        case 0xF2: { /* graph_set_field_index(program, node_id, idx) → program */
            if (nargs < 3) return args[0];
            iris_graph_t *g = (args[0] && args[0]->tag == IRIS_PROGRAM) ? args[0]->graph : NULL;
            if (!g) return args[0];
            iris_node_t *n = iris_graph_raw_find_node(g, (uint64_t)iris_as_int(args[1]));
            if (n && n->kind == NK_PROJECT) n->payload.field_index = (uint16_t)iris_as_int(args[2]);
            return args[0];
        }

        /* String operations */
        case 0xB0: return prim_str_len(args, nargs);
        case 0xB1: return prim_str_concat(args, nargs);
        case 0xB6: return prim_str_to_int(args, nargs);
        case 0xB7: return prim_int_to_string(args, nargs);

        /* Math */
        case 0xD8: return prim_math_sqrt(args, nargs);
        case 0xD9: return prim_math_log(args, nargs);
        case 0xDA: return prim_math_exp(args, nargs);

        /* Print */
        case 0xF4: return prim_print(args, nargs);

        default:
            fprintf(stderr, "warning: unknown prim opcode 0x%02x\n", opcode);
            return iris_unit();
    }
}

/* -----------------------------------------------------------------------
 * Named wrappers (called by generated iris_interp_compiled.c)
 * ----------------------------------------------------------------------- */

#define WRAP2(name, op) \
    iris_value_t *name(iris_value_t *a, iris_value_t *b) { \
        iris_value_t *args[2] = {a, b}; \
        return iris_eval_prim(op, args, 2, NULL); \
    }
#define WRAP1(name, op) \
    iris_value_t *name(iris_value_t *a) { \
        iris_value_t *args[1] = {a}; \
        return iris_eval_prim(op, args, 1, NULL); \
    }

WRAP2(iris_add, 0x00)
WRAP2(iris_sub, 0x01)
WRAP2(iris_mul, 0x02)
WRAP2(iris_div, 0x03)
WRAP2(iris_mod, 0x04)
WRAP1(iris_neg, 0x05)
WRAP1(iris_abs_val, 0x06)
WRAP2(iris_min, 0x07)
WRAP2(iris_max, 0x08)
WRAP2(iris_pow, 0x09)

WRAP2(iris_eq, 0x20)
WRAP2(iris_ne, 0x21)
WRAP2(iris_lt, 0x22)
WRAP2(iris_gt, 0x23)
WRAP2(iris_le, 0x24)
WRAP2(iris_ge, 0x25)

WRAP2(iris_and, 0x26)
WRAP2(iris_or,  0x27)
WRAP1(iris_not, 0x28)

WRAP2(iris_str_concat, 0xB1)
WRAP1(iris_str_len, 0xB0)
WRAP1(iris_str_to_int, 0xB6)
WRAP1(iris_int_to_string, 0xB7)
WRAP2(iris_list_append, 0xC5)

iris_value_t *iris_tuple_get(iris_value_t *tup, iris_value_t *idx) {
    iris_value_t *args[2] = {tup, idx};
    return iris_eval_prim(0xD2, args, 2, NULL);
}

iris_value_t *iris_tuple_len_val(iris_value_t *v) {
    iris_value_t *args[1] = {v};
    return iris_eval_prim(0xD6, args, 1, NULL);
}

#undef WRAP2
#undef WRAP1
