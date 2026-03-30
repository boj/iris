/*
<<<<<<< HEAD
 * iris_eval.c — Tree-walking evaluator for SemanticGraph programs
 *
 * Recursively evaluates nodes by kind: Lit, Prim, Guard, Lambda/Apply,
 * Let, Fold, Tuple, Inject, Project, Match, Effect, etc.
 *
 * The evaluator maintains an environment (binder_id -> value) and a
 * step counter to bound execution.
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>

#include "iris_eval.h"
#include "iris_json.h"

/* -----------------------------------------------------------------------
 * Environment implementation
 * ----------------------------------------------------------------------- */

iris_env_t *iris_env_new(void) {
    iris_env_t *e = (iris_env_t *)calloc(1, sizeof(iris_env_t));
    e->cap = 16;
    e->entries = (iris_env_entry_t *)calloc(e->cap, sizeof(iris_env_entry_t));
    return e;
}

void iris_env_free(iris_env_t *env) {
    if (!env) return;
    for (size_t i = 0; i < env->len; i++) {
        iris_release(env->entries[i].value);
    }
    free(env->entries);
    free(env);
}

void iris_env_set(iris_env_t *env, uint32_t binder, iris_value_t *val) {
    /* Update existing? */
    for (size_t i = 0; i < env->len; i++) {
        if (env->entries[i].binder == binder) {
            iris_release(env->entries[i].value);
            env->entries[i].value = val;
            iris_retain(val);
            return;
        }
    }
    /* Append */
    if (env->len >= env->cap) {
        env->cap *= 2;
        env->entries = (iris_env_entry_t *)realloc(env->entries,
            env->cap * sizeof(iris_env_entry_t));
    }
    env->entries[env->len].binder = binder;
    env->entries[env->len].value = val;
    iris_retain(val);
    env->len++;
}

iris_value_t *iris_env_get(iris_env_t *env, uint32_t binder) {
    if (!env) return NULL;
    for (size_t i = 0; i < env->len; i++) {
        if (env->entries[i].binder == binder) {
            iris_retain(env->entries[i].value);
            return env->entries[i].value;
        }
    }
    return NULL;
}

iris_env_t *iris_env_clone(iris_env_t *env) {
    iris_env_t *c = iris_env_new();
    if (!env) return c;
    for (size_t i = 0; i < env->len; i++) {
        iris_env_set(c, env->entries[i].binder, env->entries[i].value);
    }
    return c;
}

/* -----------------------------------------------------------------------
 * Lit evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_lit(iris_eval_ctx_t *ctx, iris_node_t *node) {
    uint8_t type_tag = node->payload.lit.type_tag;
    uint8_t *value = node->payload.lit.value;
    size_t vlen = node->payload.lit.value_len;

    switch (type_tag) {
    case 0x00: { /* Int */
        if (vlen >= 8) {
            int64_t n;
            memcpy(&n, value, 8); /* little-endian */
            return iris_int(n);
        }
        return iris_int(0);
    }
    case 0x01: { /* Nat -> Int */
        if (vlen >= 8) {
            uint64_t n;
            memcpy(&n, value, 8);
            return iris_int((int64_t)n);
        }
        return iris_int(0);
    }
    case 0x02: { /* Float64 */
        if (vlen >= 8) {
            double f;
            memcpy(&f, value, 8);
            return iris_f64(f);
        }
        return iris_f64(0.0);
    }
    case 0x03: { /* Float32 -> Float64 */
        if (vlen >= 4) {
            float f;
            memcpy(&f, value, 4);
            return iris_f64((double)f);
        }
        return iris_f64(0.0);
    }
    case 0x04: { /* Bool */
        if (vlen >= 1) return iris_bool(value[0] != 0);
        return iris_bool(0);
    }
    case 0x05: { /* Bytes */
        return iris_bytes(value, vlen);
    }
    case 0x06: { /* Unit */
        return iris_unit();
    }
    case 0x07: { /* String */
        return iris_string_len((const char *)value, vlen);
    }
    case 0xFF: { /* Input reference */
        if (vlen > 0) {
            uint32_t index = value[0];
            uint32_t binder = 0xFFFF0000u + index;
            iris_value_t *v = iris_env_get(ctx->env, binder);
            if (v) return v;
        }
        return iris_unit();
    }
    default:
        fprintf(stderr, "eval_lit: unknown type_tag 0x%02x\n", type_tag);
        return iris_unit();
    }
}

/* -----------------------------------------------------------------------
 * Guard evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_guard(iris_eval_ctx_t *ctx, iris_node_t *node,
                                uint32_t depth) {
    uint64_t pred_id = node->payload.guard.pred;
    uint64_t body_id = node->payload.guard.body;
    uint64_t fall_id = node->payload.guard.fallback;

    iris_value_t *pred = iris_eval_node(ctx, pred_id, depth + 1);
    int truthy = iris_is_truthy(pred);
    iris_release(pred);

    if (truthy) {
        return iris_eval_node(ctx, body_id, depth + 1);
    } else {
        return iris_eval_node(ctx, fall_id, depth + 1);
    }
}

/* -----------------------------------------------------------------------
 * Tuple evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_tuple(iris_eval_ctx_t *ctx, uint64_t node_id,
                                uint32_t depth) {
    uint64_t targets[32];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 32);

    iris_value_t **elems = NULL;
    if (n > 0) {
        elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
        for (size_t i = 0; i < n; i++) {
            elems[i] = iris_eval_node(ctx, targets[i], depth + 1);
        }
    }
    iris_value_t *result = iris_tuple(elems, n);

    /* Release our temporary refs (tuple retains them) */
    for (size_t i = 0; i < n; i++) iris_release(elems[i]);
    free(elems);
    return result;
}

/* -----------------------------------------------------------------------
 * Inject evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_inject(iris_eval_ctx_t *ctx, iris_node_t *node,
                                 uint64_t node_id, uint32_t depth) {
    uint16_t tag = node->payload.inject.tag_index;
    uint64_t targets[4];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 4);
    iris_value_t *inner;
    if (n > 0) {
        inner = iris_eval_node(ctx, targets[0], depth + 1);
    } else {
        inner = iris_unit();
    }
    iris_value_t *result = iris_tagged(tag, inner);
    iris_release(inner);
    return result;
}

/* -----------------------------------------------------------------------
 * Project evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_project(iris_eval_ctx_t *ctx, iris_node_t *node,
                                  uint64_t node_id, uint32_t depth) {
    uint16_t fi = node->payload.project.field_index;
    uint64_t targets[4];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 4);
    if (n == 0) return iris_unit();

    iris_value_t *val = iris_eval_node(ctx, targets[0], depth + 1);
    if (val->type != IRIS_TUPLE) {
        fprintf(stderr, "project: expected Tuple, got type %d\n", val->type);
        iris_release(val);
        return iris_unit();
    }
    iris_value_t *result;
    if ((size_t)fi < val->tuple.len) {
        result = val->tuple.elems[fi];
        iris_retain(result);
    } else {
        result = iris_unit();
    }
    iris_release(val);
    return result;
}

/* -----------------------------------------------------------------------
 * Match evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_match(iris_eval_ctx_t *ctx, uint64_t node_id,
                                uint32_t depth) {
    /* Scrutinee is on edge label Scrutinee */
    uint64_t scr_id = iris_graph_edge_target(ctx->graph, node_id,
                                              0, EL_SCRUTINEE);
    if (scr_id == 0) return iris_unit();

    iris_value_t *scr = iris_eval_node(ctx, scr_id, depth + 1);

    /* Get arms as Argument edges sorted by port */
    uint64_t targets[32];
    size_t narms = iris_graph_argument_targets(ctx->graph, node_id, targets, 32);

    if (scr->type == IRIS_TAGGED && narms > 0) {
        uint16_t tag = scr->tagged.tag;
        /* Arm index = tag */
        if ((size_t)tag < narms) {
            iris_value_t *payload = scr->tagged.payload;
            iris_retain(payload);
            iris_release(scr);
            /* Evaluate the arm, with the payload available somehow */
            /* For now: bind to a well-known binder */
            iris_env_set(ctx->env, 0xFFFE0000u + tag, payload);
            iris_value_t *result = iris_eval_node(ctx, targets[tag], depth + 1);
            iris_release(payload);
            return result;
        }
    }

    /* Fallback: evaluate first arm */
    iris_release(scr);
    if (narms > 0) return iris_eval_node(ctx, targets[0], depth + 1);
    return iris_unit();
}

/* -----------------------------------------------------------------------
 * Let evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_let(iris_eval_ctx_t *ctx, uint64_t node_id,
                              uint32_t depth) {
    /* Binding edge: value to bind */
    uint64_t bind_target = iris_graph_edge_target(ctx->graph, node_id,
                                                   0, EL_BINDING);
    /* Continuation edge: body */
    uint64_t cont_target = iris_graph_edge_target(ctx->graph, node_id,
                                                   0, EL_CONTINUATION);
    if (bind_target == 0 || cont_target == 0) return iris_unit();

    /* Argument edge (port 0) = the expression to bind */
    uint64_t arg_targets[4];
    size_t nargs = iris_graph_argument_targets(ctx->graph, node_id, arg_targets, 4);

    iris_value_t *val;
    if (nargs > 0) {
        val = iris_eval_node(ctx, arg_targets[0], depth + 1);
    } else {
        val = iris_unit();
    }

    /* The binding target is a node; we need to find what binder ID it refers to.
     * In IRIS, Let nodes bind to a Lambda binder. The bind_target is the binder
     * node. We use its NodeId as the binder key (simple approach). */
    iris_node_t *bind_node = iris_graph_find_node(ctx->graph, bind_target);
    uint32_t binder;
    if (bind_node && bind_node->kind == NK_LAMBDA) {
        binder = bind_node->payload.lambda.binder_id;
    } else {
        /* Use the node ID low bits as binder */
        binder = (uint32_t)(bind_target & 0xFFFFFFFF);
    }

    iris_env_set(ctx->env, binder, val);
    iris_release(val);

    return iris_eval_node(ctx, cont_target, depth + 1);
}

/* -----------------------------------------------------------------------
 * Lambda evaluation (returns a thunk-like value, for now just the body)
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_lambda(iris_eval_ctx_t *ctx, uint64_t node_id,
                                 iris_node_t *node, uint32_t depth) {
    /* For the bootstrap interpreter, Lambdas in interpreter.json are not
     * common (the interpreter is Lit/Prim/Guard only). When we encounter
     * a Lambda, we need to capture the environment and return a closure.
     * For simplicity, we represent closures as a Program value wrapping
     * a graph with modified root. */
    (void)depth;
    (void)node;

    /* Create a new graph wrapper whose root is this lambda */
    iris_graph_t *closure_graph = iris_graph_set_root(ctx->graph, node_id);
    iris_value_t *result = iris_program(closure_graph);
    iris_graph_release(closure_graph);
    return result;
}

/* -----------------------------------------------------------------------
 * Apply evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_apply(iris_eval_ctx_t *ctx, uint64_t node_id,
                                uint32_t depth) {
    uint64_t targets[32];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 32);

    if (n == 0) return iris_unit();

    /* First argument is the function */
    iris_value_t *func = iris_eval_node(ctx, targets[0], depth + 1);

    /* Evaluate remaining arguments */
    size_t nargs = n - 1;
    iris_value_t **args = NULL;
    if (nargs > 0) {
        args = (iris_value_t **)malloc(sizeof(iris_value_t *) * nargs);
        for (size_t i = 0; i < nargs; i++) {
            args[i] = iris_eval_node(ctx, targets[i + 1], depth + 1);
        }
    }

    iris_value_t *result;
    if (func->type == IRIS_PROGRAM && func->graph) {
        /* Apply: evaluate the closure's graph with arguments bound */
        iris_graph_t *fg = func->graph;
        iris_node_t *root_node = iris_graph_find_node(fg, fg->root);

        if (root_node && root_node->kind == NK_LAMBDA) {
            /* Bind the argument to the lambda's binder */
            iris_eval_ctx_t sub_ctx;
            sub_ctx.graph = fg;
            sub_ctx.env = iris_env_clone(ctx->env);
            sub_ctx.step_count = ctx->step_count;
            sub_ctx.max_steps = ctx->max_steps;

            uint32_t binder = root_node->payload.lambda.binder_id;
            if (nargs > 0) {
                iris_env_set(sub_ctx.env, binder, args[0]);
            }

            /* Find continuation (body) */
            uint64_t body_id = iris_graph_edge_target(fg, fg->root,
                                                       0, EL_CONTINUATION);
            if (body_id != 0) {
                result = iris_eval_node(&sub_ctx, body_id, depth + 1);
            } else {
                result = iris_unit();
            }
            ctx->step_count = sub_ctx.step_count;
            iris_env_free(sub_ctx.env);
        } else {
            /* Not a lambda root — just eval the graph */
            result = iris_eval(fg, nargs > 0 ? args[0] : iris_unit());
        }
    } else {
        /* Cannot apply a non-program value */
        result = iris_unit();
    }

    iris_release(func);
    for (size_t i = 0; i < nargs; i++) iris_release(args[i]);
    free(args);
    return result;
}

/* -----------------------------------------------------------------------
 * Fold evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_fold(iris_eval_ctx_t *ctx, uint64_t node_id,
                               uint32_t depth) {
    uint64_t targets[32];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 32);

    /* Fold(init, list, step):
     * arg0 = step function, arg1 = init, arg2 = list (or vice versa) */
    if (n < 2) return iris_unit();

    /* Evaluate all arguments */
    iris_value_t **args = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) {
        args[i] = iris_eval_node(ctx, targets[i], depth + 1);
    }

    iris_value_t *result;
    if (n >= 3 && args[2]->type == IRIS_TUPLE) {
        /* fold(step_fn, init, list) */
        iris_value_t *acc = args[1];
        iris_retain(acc);
        iris_value_t *list = args[2];
        size_t len = list->tuple.len;

        for (size_t i = 0; i < len; i++) {
            iris_value_t *elem = list->tuple.elems[i];
            iris_retain(elem);

            /* Build input tuple: (acc, elem) */
            iris_value_t *pair_elems[2] = { acc, elem };
            iris_value_t *pair = iris_tuple(pair_elems, 2);

            /* Evaluate step function with (acc, elem) as input */
            if (args[0]->type == IRIS_PROGRAM) {
                iris_value_t *new_acc = iris_eval(args[0]->graph, pair);
                iris_release(pair);
                iris_release(acc);
                iris_release(elem);
                acc = new_acc;
            } else {
                iris_release(pair);
                iris_release(elem);
                break;
            }
        }
        result = acc;
    } else if (n >= 2 && args[1]->type == IRIS_TUPLE) {
        /* fold(init, list) — the fold body is in a Continuation edge */
        iris_value_t *acc = args[0];
        iris_retain(acc);
        iris_value_t *list = args[1];
        size_t len = list->tuple.len;

        uint64_t body_id = iris_graph_edge_target(ctx->graph, node_id,
                                                   0, EL_CONTINUATION);
        if (body_id == 0) { result = acc; goto fold_done; }

        for (size_t i = 0; i < len; i++) {
            iris_value_t *elem = list->tuple.elems[i];
            iris_retain(elem);

            /* Bind acc and elem to input binders */
            iris_env_set(ctx->env, 0xFFFF0000u, acc);
            iris_env_set(ctx->env, 0xFFFF0001u, elem);

            iris_value_t *new_acc = iris_eval_node(ctx, body_id, depth + 1);
            iris_release(acc);
            iris_release(elem);
            acc = new_acc;
        }
        result = acc;
    } else {
        result = n > 0 ? args[0] : iris_unit();
        iris_retain(result);
    }

fold_done:
    for (size_t i = 0; i < n; i++) iris_release(args[i]);
    free(args);
    return result;
}

/* -----------------------------------------------------------------------
 * Effect evaluation
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_effect(iris_eval_ctx_t *ctx, iris_node_t *node,
                                 uint64_t node_id, uint32_t depth) {
    uint8_t effect_tag = node->payload.effect_tag;
    uint64_t targets[16];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 16);

    iris_value_t **args = NULL;
    if (n > 0) {
        args = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
        for (size_t i = 0; i < n; i++) {
            args[i] = iris_eval_node(ctx, targets[i], depth + 1);
        }
    }

    iris_value_t *result = iris_dispatch_effect(effect_tag, args, n);

    for (size_t i = 0; i < n; i++) iris_release(args[i]);
    free(args);
    return result;
}

/* -----------------------------------------------------------------------
 * Prim evaluation (eagerly evaluate args, then dispatch)
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_prim(iris_eval_ctx_t *ctx, iris_node_t *node,
                               uint64_t node_id, uint32_t depth) {
    uint8_t opcode = node->payload.prim_opcode;

    uint64_t targets[32];
    size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 32);

    iris_value_t **args = NULL;
    if (n > 0) {
        args = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
        for (size_t i = 0; i < n; i++) {
            args[i] = iris_eval_node(ctx, targets[i], depth + 1);
        }
    }

    iris_value_t *result = iris_dispatch_prim(ctx, opcode, args, n);

    for (size_t i = 0; i < n; i++) iris_release(args[i]);
    free(args);
    return result;
}

/* -----------------------------------------------------------------------
 * Main eval dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_node(iris_eval_ctx_t *ctx, uint64_t node_id,
                             uint32_t depth) {
    if (depth > IRIS_MAX_DEPTH) {
        fprintf(stderr, "iris: recursion depth exceeded (%u)\n", depth);
        return iris_unit();
    }

    ctx->step_count++;
    if (ctx->step_count > ctx->max_steps) {
        fprintf(stderr, "iris: step limit exceeded (%lu)\n",
                (unsigned long)ctx->max_steps);
        return iris_unit();
    }

    iris_node_t *node = iris_graph_find_node(ctx->graph, node_id);
    if (!node) {
        fprintf(stderr, "iris: missing node %lu\n", (unsigned long)node_id);
=======
 * iris_eval.c -- Tree-walking evaluator for IRIS SemanticGraphs.
 *
 * Evaluates a graph starting from its root node. Supports:
 *   - Lit (constants, input references via type_tag 0xFF)
 *   - Prim (dispatches through iris_eval_prim)
 *   - Guard (predicate ? body : fallback)
 *   - Tuple (construct product values)
 *   - Project (extract tuple field)
 *   - Fold (iterative evaluation)
 *   - Lambda/Apply (first-class functions via closures)
 *   - Let (local bindings)
 *   - Inject (tagged union construction)
 *   - Match (pattern matching on tagged values)
 *
 * The evaluator passes a single `input` value threaded through the graph.
 * InputRef nodes (Lit with type_tag=0xFF) extract parameters from a tuple input.
 */

#include "iris_rt.h"
#include "iris_graph.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* -----------------------------------------------------------------------
 * Evaluation limits
 * ----------------------------------------------------------------------- */

#define MAX_EVAL_DEPTH 256
#define MAX_EVAL_STEPS 500000

/* -----------------------------------------------------------------------
 * Evaluation context (threaded through recursive calls)
 * ----------------------------------------------------------------------- */

typedef struct {
    iris_graph_t   *graph;
    iris_value_t   *input;
    iris_value_t   *self_program;   /* Value::Program(self) for opcode 0x80 */
    uint32_t        depth;
    uint64_t        steps;
} eval_ctx_t;

/* Forward declaration */
static iris_value_t *eval_node(eval_ctx_t *ctx, uint64_t node_id);

/* -----------------------------------------------------------------------
 * Collect argument targets sorted by port
 * ----------------------------------------------------------------------- */

static uint32_t collect_args(const iris_graph_t *g, uint64_t source,
                             uint64_t *out, uint32_t out_cap) {
    return iris_graph_raw_outgoing(g, source, EL_ARGUMENT, out, out_cap);
}

/* Collect continuation target (label=3) */
static int find_continuation(const iris_graph_t *g, uint64_t source, uint64_t *out) {
    for (uint32_t i = 0; i < g->edge_count; i++) {
        const iris_edge_t *e = &g->edges[i];
        if (e->source == source && e->label == EL_CONTINUATION) {
            *out = e->target;
            return 1;
        }
    }
    return 0;
}

/* Collect binding target (label=2) */
static int find_binding(const iris_graph_t *g, uint64_t source, uint64_t *out) {
    for (uint32_t i = 0; i < g->edge_count; i++) {
        const iris_edge_t *e = &g->edges[i];
        if (e->source == source && e->label == EL_BINDING) {
            *out = e->target;
            return 1;
        }
    }
    return 0;
}

/* -----------------------------------------------------------------------
 * Evaluate a single node
 * ----------------------------------------------------------------------- */

static iris_value_t *eval_node(eval_ctx_t *ctx, uint64_t node_id) {
    if (ctx->depth > MAX_EVAL_DEPTH) {
        fprintf(stderr, "error: recursion depth exceeded at node %lu\n",
                (unsigned long)node_id);
        return iris_unit();
    }
    if (ctx->steps > MAX_EVAL_STEPS) {
        fprintf(stderr, "error: step limit exceeded at node %lu\n",
                (unsigned long)node_id);
        return iris_unit();
    }
    ctx->steps++;

    iris_node_t *node = iris_graph_raw_find_node(ctx->graph, node_id);
    if (!node) {
        fprintf(stderr, "error: node %lu not found\n", (unsigned long)node_id);
>>>>>>> worktree-agent-a01b95aa
        return iris_unit();
    }

    switch (node->kind) {
<<<<<<< HEAD
    case NK_LIT:
        return eval_lit(ctx, node);
    case NK_PRIM:
        return eval_prim(ctx, node, node_id, depth);
    case NK_GUARD:
        return eval_guard(ctx, node, depth);
    case NK_TUPLE:
        return eval_tuple(ctx, node_id, depth);
    case NK_INJECT:
        return eval_inject(ctx, node, node_id, depth);
    case NK_PROJECT:
        return eval_project(ctx, node, node_id, depth);
    case NK_MATCH:
        return eval_match(ctx, node_id, depth);
    case NK_LET:
        return eval_let(ctx, node_id, depth);
    case NK_LAMBDA:
        return eval_lambda(ctx, node_id, node, depth);
    case NK_APPLY:
        return eval_apply(ctx, node_id, depth);
    case NK_FOLD:
        return eval_fold(ctx, node_id, depth);
    case NK_EFFECT:
        return eval_effect(ctx, node, node_id, depth);
    case NK_REWRITE: {
        /* Transparent: evaluate body directly */
        uint64_t targets[4];
        size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 4);
        if (n > 0) return iris_eval_node(ctx, targets[0], depth + 1);
        return iris_unit();
    }
    case NK_TYPEABST:
    case NK_TYPEAPP: {
        /* Type erasure: evaluate the single argument */
        uint64_t targets[4];
        size_t n = iris_graph_argument_targets(ctx->graph, node_id, targets, 4);
        if (n > 0) return iris_eval_node(ctx, targets[0], depth + 1);
        return iris_unit();
    }
    default:
        fprintf(stderr, "iris: unsupported node kind 0x%02x\n", node->kind);
=======
    case NK_LIT: {
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
            case 0xFF: { /* InputRef */
                uint8_t param_idx = (lit->value_len >= 1) ? lit->value[0] : 0;
                iris_value_t *inp = ctx->input;
                if (!inp) return iris_unit();
                /* If input is a tuple, extract the param_idx-th element */
                if (inp->tag == IRIS_TUPLE) {
                    if (param_idx < inp->tuple.len) {
                        return inp->tuple.elems[param_idx];
                    }
                    return iris_unit();
                }
                /* If param_idx == 0 and input is scalar, return it directly */
                if (param_idx == 0) return inp;
                return iris_unit();
            }
            default:
                return iris_unit();
        }
    }

    case NK_PRIM: {
        /* Evaluate arguments, then dispatch */
        uint64_t arg_ids[32];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 32);

        iris_value_t *args[32];
        ctx->depth++;
        for (uint32_t i = 0; i < nargs; i++) {
            args[i] = eval_node(ctx, arg_ids[i]);
        }
        ctx->depth--;

        return iris_eval_prim(node->payload.prim_opcode, args, nargs,
                              ctx->self_program);
    }

    case NK_GUARD: {
        /* Evaluate predicate; if truthy, evaluate body; else fallback */
        iris_guard_payload_t *g = &node->payload.guard;

        ctx->depth++;
        iris_value_t *pred = eval_node(ctx, g->predicate_node);
        ctx->depth--;

        bool cond = false;
        if (pred->tag == IRIS_BOOL)     cond = pred->b;
        else if (pred->tag == IRIS_INT) cond = (pred->i != 0);

        ctx->depth++;
        iris_value_t *result;
        if (cond) {
            result = eval_node(ctx, g->body_node);
        } else {
            result = eval_node(ctx, g->fallback_node);
        }
        ctx->depth--;
        return result;
    }

    case NK_TUPLE: {
        uint64_t arg_ids[64];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 64);

        iris_value_t *elems[64];
        ctx->depth++;
        for (uint32_t i = 0; i < nargs; i++) {
            elems[i] = eval_node(ctx, arg_ids[i]);
        }
        ctx->depth--;

        /* Build tuple from evaluated elements */
        iris_value_t **heap_elems = NULL;
        if (nargs > 0) {
            /* Use arena allocation */
            heap_elems = malloc(nargs * sizeof(iris_value_t *));
            memcpy(heap_elems, elems, nargs * sizeof(iris_value_t *));
        }
        return iris_tuple(heap_elems, nargs);
    }

    case NK_PROJECT: {
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);
        if (nargs < 1) return iris_unit();

        ctx->depth++;
        iris_value_t *val = eval_node(ctx, arg_ids[0]);
        ctx->depth--;

        uint16_t fi = node->payload.field_index;
        if (val->tag == IRIS_TUPLE && fi < val->tuple.len) {
            return val->tuple.elems[fi];
        }
        return iris_unit();
    }

    case NK_FOLD: {
        /*
         * Fold: iterative evaluation.
         * Structure: fold has an Argument edge (port 0) to the initial accumulator,
         * a Continuation edge to the step function body,
         * and a Binding edge to a "condition" or "seed" node.
         *
         * Simplified: evaluate the body with (accumulator, input) on each step.
         */
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);

        uint64_t cont_id;
        if (!find_continuation(ctx->graph, node_id, &cont_id)) {
            /* No continuation: just return the argument */
            if (nargs > 0) {
                ctx->depth++;
                iris_value_t *r = eval_node(ctx, arg_ids[0]);
                ctx->depth--;
                return r;
            }
            return iris_unit();
        }

        /* Evaluate initial accumulator */
        iris_value_t *acc = iris_unit();
        if (nargs > 0) {
            ctx->depth++;
            acc = eval_node(ctx, arg_ids[0]);
            ctx->depth--;
        }

        /* Iteratively evaluate the continuation with (acc, input) */
        for (int iter = 0; iter < 1000; iter++) {
            /* Build a tuple (acc, input) as the input for the step body */
            iris_value_t *step_elems[2] = { acc, ctx->input };
            iris_value_t *step_input = iris_tuple(step_elems, 2);

            eval_ctx_t step_ctx = *ctx;
            step_ctx.input = step_input;
            step_ctx.depth = ctx->depth + 1;

            iris_value_t *result = eval_node(&step_ctx, cont_id);
            ctx->steps = step_ctx.steps;

            /* If result is a tuple (tag, value), check for termination */
            if (result->tag == IRIS_TAGGED) {
                /* Tagged(0, payload) = continue, Tagged(1, payload) = done */
                if (result->tagged.tag_index == 1) {
                    return result->tagged.payload;
                }
                acc = result->tagged.payload;
            } else if (result->tag == IRIS_TUPLE && result->tuple.len == 2) {
                /* Convention: (0, val) = continue, (1, val) = done */
                iris_value_t *tag_val = result->tuple.elems[0];
                if (tag_val && tag_val->tag == IRIS_BOOL && tag_val->b) {
                    return result->tuple.elems[1];
                }
                if (tag_val && tag_val->tag == IRIS_INT && tag_val->i != 0) {
                    return result->tuple.elems[1];
                }
                acc = result->tuple.elems[1];
            } else {
                return result;
            }
        }
        return acc;
    }

    case NK_LAMBDA: {
        /*
         * Lambda evaluation: look for an Apply parent.
         * In the simple case (direct evaluation of a lambda), we skip it
         * and evaluate the continuation body.
         */
        uint64_t cont_id;
        if (find_continuation(ctx->graph, node_id, &cont_id)) {
            ctx->depth++;
            iris_value_t *result = eval_node(ctx, cont_id);
            ctx->depth--;
            return result;
        }
        return iris_unit();
    }

    case NK_APPLY: {
        /*
         * Apply: evaluate function (port 0) and argument (port 1+).
         * For the bootstrap interpreter, the function is typically a graph node
         * reference that we evaluate.
         */
        uint64_t arg_ids[16];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 16);
        if (nargs < 1) return iris_unit();

        ctx->depth++;
        /* Evaluate all arguments */
        iris_value_t *vals[16];
        for (uint32_t i = 0; i < nargs; i++) {
            vals[i] = eval_node(ctx, arg_ids[i]);
        }
        ctx->depth--;

        /* If the function is a program, evaluate it with the remaining args as input */
        if (vals[0]->tag == IRIS_PROGRAM) {
            iris_value_t *input;
            if (nargs == 2) {
                input = vals[1];
            } else if (nargs > 2) {
                input = iris_tuple(&vals[1], nargs - 1);
            } else {
                input = iris_unit();
            }
            return iris_graph_eval(vals[0], input);
        }

        /* Otherwise, return the function value (already evaluated) */
        return vals[0];
    }

    case NK_LET: {
        /*
         * Let: binding edge = value to bind, continuation edge = body.
         * Evaluate binding, then evaluate body with the binding available.
         */
        uint64_t bind_id, cont_id;
        int have_bind = find_binding(ctx->graph, node_id, &bind_id);
        int have_cont = find_continuation(ctx->graph, node_id, &cont_id);

        if (have_bind) {
            ctx->depth++;
            /* Evaluate the binding (side effect: available via input) */
            eval_node(ctx, bind_id);
            ctx->depth--;
        }

        if (have_cont) {
            ctx->depth++;
            iris_value_t *result = eval_node(ctx, cont_id);
            ctx->depth--;
            return result;
        }

        /* Fallback: evaluate arguments */
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);
        if (nargs > 0) {
            ctx->depth++;
            iris_value_t *r = eval_node(ctx, arg_ids[nargs - 1]);
            ctx->depth--;
            return r;
        }
        return iris_unit();
    }

    case NK_INJECT: {
        /* Construct a tagged value: Inject(tag_index, argument) */
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);
        iris_value_t *payload = iris_unit();
        if (nargs > 0) {
            ctx->depth++;
            payload = eval_node(ctx, arg_ids[0]);
            ctx->depth--;
        }
        return iris_tagged(node->payload.tag_index, payload);
    }

    case NK_MATCH: {
        /* Simple match: evaluate scrutinee, then dispatch on tag */
        uint64_t arg_ids[32];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 32);
        if (nargs < 1) return iris_unit();

        /* First argument is typically the scrutinee */
        ctx->depth++;
        iris_value_t *scrutinee = eval_node(ctx, arg_ids[0]);
        ctx->depth--;

        /* If scrutinee is tagged, use tag as index into remaining arms */
        if (scrutinee->tag == IRIS_TAGGED) {
            uint16_t tag = scrutinee->tagged.tag_index;
            uint32_t arm_idx = tag + 1; /* arms start at port 1 */
            if (arm_idx < nargs) {
                eval_ctx_t arm_ctx = *ctx;
                arm_ctx.input = scrutinee->tagged.payload;
                arm_ctx.depth = ctx->depth + 1;
                return eval_node(&arm_ctx, arg_ids[arm_idx]);
            }
        }

        /* Fallback: evaluate first arm */
        if (nargs > 1) {
            ctx->depth++;
            iris_value_t *r = eval_node(ctx, arg_ids[1]);
            ctx->depth--;
            return r;
        }
        return iris_unit();
    }

    case NK_EFFECT: {
        /* Evaluate arguments, then perform the effect */
        uint64_t arg_ids[16];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 16);

        iris_value_t *args[16];
        ctx->depth++;
        for (uint32_t i = 0; i < nargs; i++) {
            args[i] = eval_node(ctx, arg_ids[i]);
        }
        ctx->depth--;

        iris_value_t *tag = iris_int((int64_t)node->payload.effect_tag);
        iris_value_t *arg_tuple = iris_tuple(args, nargs);
        return iris_perform_effect(tag, arg_tuple);
    }

    case NK_REF:
    case NK_REWRITE:
    case NK_LET_REC:
    case NK_TYPE_ABST:
    case NK_TYPE_APP: {
        /* These need continuation/argument chasing */
        uint64_t cont_id;
        if (find_continuation(ctx->graph, node_id, &cont_id)) {
            ctx->depth++;
            iris_value_t *r = eval_node(ctx, cont_id);
            ctx->depth--;
            return r;
        }
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);
        if (nargs > 0) {
            ctx->depth++;
            iris_value_t *r = eval_node(ctx, arg_ids[0]);
            ctx->depth--;
            return r;
        }
        return iris_unit();
    }

    default:
        fprintf(stderr, "warning: unsupported node kind 0x%02x at node %lu\n",
                node->kind, (unsigned long)node_id);
>>>>>>> worktree-agent-a01b95aa
        return iris_unit();
    }
}

/* -----------------------------------------------------------------------
<<<<<<< HEAD
 * Public entry points
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval(iris_graph_t *graph, iris_value_t *inputs) {
    iris_eval_ctx_t ctx;
    ctx.graph = graph;
    ctx.env = iris_env_new();
    ctx.step_count = 0;
    ctx.max_steps = IRIS_MAX_STEPS;

    /* Bind inputs: if inputs is a tuple, bind each element to binder 0xFFFF0000+i.
     * If inputs is a scalar, bind to binder 0xFFFF0000. */
    if (inputs) {
        if (inputs->type == IRIS_TUPLE) {
            for (size_t i = 0; i < inputs->tuple.len; i++) {
                iris_env_set(ctx.env, 0xFFFF0000u + (uint32_t)i,
                            inputs->tuple.elems[i]);
            }
        } else {
            iris_env_set(ctx.env, 0xFFFF0000u, inputs);
        }
    }

    iris_value_t *result = iris_eval_node(&ctx, graph->root, 0);
    iris_env_free(ctx.env);
    return result;
}

iris_value_t *iris_eval_env(iris_graph_t *graph, uint32_t binder,
                            iris_value_t *value, iris_value_t *inputs) {
    iris_eval_ctx_t ctx;
    ctx.graph = graph;
    ctx.env = iris_env_new();
    ctx.step_count = 0;
    ctx.max_steps = IRIS_MAX_STEPS;

    /* Bind inputs */
    if (inputs) {
        if (inputs->type == IRIS_TUPLE) {
            for (size_t i = 0; i < inputs->tuple.len; i++) {
                iris_env_set(ctx.env, 0xFFFF0000u + (uint32_t)i,
                            inputs->tuple.elems[i]);
            }
        } else {
            iris_env_set(ctx.env, 0xFFFF0000u, inputs);
        }
    }

    /* Bind the extra binder->value */
    iris_env_set(ctx.env, binder, value);

    iris_value_t *result = iris_eval_node(&ctx, graph->root, 0);
    iris_env_free(ctx.env);
    return result;
=======
 * Public entry point
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_graph(iris_graph_t *g, iris_value_t *input) {
    if (!g) return iris_unit();

    iris_value_t *self_prog = iris_program(g);

    eval_ctx_t ctx = {
        .graph        = g,
        .input        = input ? input : iris_unit(),
        .self_program = self_prog,
        .depth        = 0,
        .steps        = 0,
    };

    return eval_node(&ctx, g->root);
}

iris_value_t *iris_eval_node(iris_graph_t *g, uint64_t node_id,
                             iris_value_t *input, uint32_t depth) {
    if (!g) return iris_unit();

    iris_value_t *self_prog = iris_program(g);

    eval_ctx_t ctx = {
        .graph        = g,
        .input        = input ? input : iris_unit(),
        .self_program = self_prog,
        .depth        = depth,
        .steps        = 0,
    };

    return eval_node(&ctx, node_id);
>>>>>>> worktree-agent-a01b95aa
}
