/*
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
#define MAX_EVAL_STEPS 5000000

/* -----------------------------------------------------------------------
 * Evaluation context (threaded through recursive calls)
 * ----------------------------------------------------------------------- */

/* Simple node memoization: cache evaluated results for DAG sharing */
#define MEMO_CAP 4096
typedef struct {
    uint64_t       key;
    iris_value_t  *val;
} memo_entry_t;

/* Binder environment: maps binder_id → value for Let/Lambda bindings */
#define ENV_CAP 256
typedef struct {
    uint32_t       binder_id;
    iris_value_t  *val;
} env_entry_t;

typedef struct {
    iris_graph_t   *graph;
    iris_value_t   *input;
    iris_value_t   *self_program;   /* Value::Program(self) for opcode 0x80 */
    uint32_t        depth;
    uint64_t        steps;
    memo_entry_t    memo[MEMO_CAP];
    uint32_t        memo_count;
    env_entry_t     env[ENV_CAP];
    uint32_t        env_count;
} eval_ctx_t;

static iris_value_t *env_get(eval_ctx_t *ctx, uint32_t binder_id) {
    /* Search from end so inner bindings shadow outer ones */
    for (int i = (int)ctx->env_count - 1; i >= 0; i--) {
        if (ctx->env[i].binder_id == binder_id) return ctx->env[i].val;
    }
    return NULL;
}

static void env_push(eval_ctx_t *ctx, uint32_t binder_id, iris_value_t *val) {
    if (ctx->env_count < ENV_CAP) {
        ctx->env[ctx->env_count].binder_id = binder_id;
        ctx->env[ctx->env_count].val = val;
        ctx->env_count++;
    }
}

static iris_value_t *memo_get(eval_ctx_t *ctx, uint64_t node_id) {
    for (uint32_t i = 0; i < ctx->memo_count; i++) {
        if (ctx->memo[i].key == node_id) return ctx->memo[i].val;
    }
    return NULL;
}

static void memo_put(eval_ctx_t *ctx, uint64_t node_id, iris_value_t *val) {
    if (ctx->memo_count < MEMO_CAP) {
        ctx->memo[ctx->memo_count].key = node_id;
        ctx->memo[ctx->memo_count].val = val;
        ctx->memo_count++;
    }
}

/* Forward declarations */
static iris_value_t *eval_node(eval_ctx_t *ctx, uint64_t node_id);
static iris_value_t *eval_node_inner(eval_ctx_t *ctx, uint64_t node_id);

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

    /* Memoize: return cached result for shared DAG nodes.
     * Skip Guard nodes (short-circuit) and impure Prim nodes (graph mutations). */
    iris_node_t *peek = iris_graph_raw_find_node(ctx->graph, node_id);
    int can_memo = peek && peek->kind != NK_GUARD;
    if (can_memo && peek->kind == NK_PRIM) {
        /* Only skip memo for impure graph mutation prims */
        uint8_t op = peek->payload.prim_opcode;
        if (op == 0x84 || op == 0x85 || op == 0x86 || op == 0x87 ||
            op == 0x88 || op == 0x8B || op == 0x8C || op == 0x8D ||
            op == 0x61 || op == 0xEE || op == 0xEF || op == 0xF1 ||
            op == 0xA1 || op == 0xF4) {
            can_memo = 0;
        }
    }
    if (can_memo) {
        iris_value_t *cached = memo_get(ctx, node_id);
        if (cached) return cached;
    }

    iris_value_t *_memo_result = eval_node_inner(ctx, node_id);
    if (can_memo) {
        memo_put(ctx, node_id, _memo_result);
    }
    return _memo_result;
}

static iris_value_t *eval_node_inner(eval_ctx_t *ctx, uint64_t node_id) {

    iris_node_t *node = iris_graph_raw_find_node(ctx->graph, node_id);
    if (!node) {
        fprintf(stderr, "error: node %lu not found\n", (unsigned long)node_id);
        return iris_unit();
    }

    switch (node->kind) {
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
            case 0xFF: { /* InputRef / BinderRef */
                /* Read the full parameter index from the value bytes */
                uint32_t param_idx = 0;
                if (lit->value_len >= 4) {
                    memcpy(&param_idx, lit->value, 4);
                } else if (lit->value_len >= 1) {
                    param_idx = lit->value[0];
                }

                /* Check if this is a binder reference (high bits set) */
                if (param_idx >= 0xFFFF0000u) {
                    uint32_t binder_id = param_idx - 0xFFFF0000u;
                    iris_value_t *bound = env_get(ctx, binder_id);
                    if (bound) return bound;
                    /* Fall through to input lookup */
                }

                /* Check raw param_idx as binder_id, and also with 0xFFFF prefix
                 * (binder IDs are 0xFFFF0000 + index, but InputRef stores just the index) */
                iris_value_t *bound = env_get(ctx, param_idx);
                if (bound) return bound;
                bound = env_get(ctx, 0xFFFF0000u + param_idx);
                if (bound) return bound;

                /* Standard input parameter lookup */
                iris_value_t *inp = ctx->input;
                if (!inp) return iris_unit();
                if (inp->tag == IRIS_TUPLE) {
                    if (param_idx < inp->tuple.len) {
                        return inp->tuple.elems[param_idx];
                    }
                    return iris_unit();
                }
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
         * Argument edges:
         *   port 0 = initial accumulator
         *   port 1 = step function (Prim opcode or Lambda)
         *   port 2 = collection (tuple to iterate over)
         */
        uint64_t arg_ids[4];
        uint32_t nargs = collect_args(ctx->graph, node_id, arg_ids, 4);

        if (nargs < 3) {
            /* Malformed fold — need acc, step, collection */
            if (nargs > 0) {
                ctx->depth++;
                iris_value_t *r = eval_node(ctx, arg_ids[0]);
                ctx->depth--;
                return r;
            }
            return iris_unit();
        }

        /* Evaluate initial accumulator (port 0) */
        ctx->depth++;
        iris_value_t *acc = eval_node(ctx, arg_ids[0]);
        ctx->depth--;

        /* Step function node (port 1) — could be Prim or Lambda */
        uint64_t step_id = arg_ids[1];
        iris_node_t *step_node = iris_graph_raw_find_node(ctx->graph, step_id);

        /* Evaluate collection (port 2) */
        ctx->depth++;
        iris_value_t *collection = eval_node(ctx, arg_ids[2]);
        ctx->depth--;

        if (!collection) return acc;

        /* Int-as-range: fold over 0..n-1 */
        if (collection->tag == IRIS_INT) {
            int64_t n = collection->i;
            if (n <= 0) return acc;
            if (n > 100000) n = 100000; /* safety limit */
            for (int64_t iter = 0; iter < n; iter++) {
                iris_value_t *elem = iris_int(iter);
                if (step_node && step_node->kind == NK_PRIM) {
                    iris_value_t *prim_args[2] = { acc, elem };
                    acc = iris_eval_prim(step_node->payload.prim_opcode,
                                         prim_args, 2, ctx->self_program);
                } else if (step_node && step_node->kind == NK_LAMBDA) {
                    uint32_t binder = step_node->payload.lambda.binder_id;
                    uint64_t body_id;
                    if (find_continuation(ctx->graph, step_id, &body_id)) {
                        iris_value_t *pair_elems[2] = { acc, elem };
                        iris_value_t *pair = iris_tuple(pair_elems, 2);
                        eval_ctx_t step_ctx = *ctx;
                        step_ctx.depth = ctx->depth + 1;
                        step_ctx.memo_count = 0;
                        env_push(&step_ctx, binder, pair);
                        acc = eval_node(&step_ctx, body_id);
                        ctx->steps = step_ctx.steps;
                    }
                } else {
                    iris_value_t *pair_elems[2] = { acc, elem };
                    iris_value_t *pair = iris_tuple(pair_elems, 2);
                    eval_ctx_t step_ctx = *ctx;
                    step_ctx.input = pair;
                    step_ctx.depth = ctx->depth + 1;
                    step_ctx.memo_count = 0;
                    acc = eval_node(&step_ctx, step_id);
                    ctx->steps = step_ctx.steps;
                }
            }
            return acc;
        }

        if (collection->tag != IRIS_TUPLE || collection->tuple.len == 0) {
            return acc;
        }

        /* Iterate over tuple collection elements */
        for (uint32_t iter = 0; iter < collection->tuple.len; iter++) {
            iris_value_t *elem = collection->tuple.elems[iter];

            if (step_node && step_node->kind == NK_PRIM) {
                /* Prim step: apply opcode directly to (acc, elem) */
                iris_value_t *prim_args[2] = { acc, elem };
                acc = iris_eval_prim(step_node->payload.prim_opcode,
                                     prim_args, 2, ctx->self_program);
            } else if (step_node && step_node->kind == NK_LAMBDA) {
                /* Lambda step: bind param to (acc, elem), eval body */
                uint32_t binder = step_node->payload.lambda.binder_id;
                uint64_t body_id;
                if (find_continuation(ctx->graph, step_id, &body_id)) {
                    iris_value_t *pair_elems[2] = { acc, elem };
                    iris_value_t *pair = iris_tuple(pair_elems, 2);

                    eval_ctx_t step_ctx = *ctx;
                    step_ctx.depth = ctx->depth + 1;
                    step_ctx.memo_count = 0;
                    env_push(&step_ctx, binder, pair);

                    acc = eval_node(&step_ctx, body_id);
                    ctx->steps = step_ctx.steps;
                } else {
                    /* Lambda without continuation — eval it as a function */
                    iris_value_t *pair_elems[2] = { acc, elem };
                    iris_value_t *pair = iris_tuple(pair_elems, 2);
                    eval_ctx_t step_ctx = *ctx;
                    step_ctx.input = pair;
                    step_ctx.depth = ctx->depth + 1;
                    step_ctx.memo_count = 0;
                    acc = eval_node(&step_ctx, step_id);
                    ctx->steps = step_ctx.steps;
                }
            } else {
                /* Unknown step function — try evaluating it as a graph */
                iris_value_t *pair_elems[2] = { acc, elem };
                iris_value_t *pair = iris_tuple(pair_elems, 2);
                eval_ctx_t step_ctx = *ctx;
                step_ctx.input = pair;
                step_ctx.depth = ctx->depth + 1;
                step_ctx.memo_count = 0;
                acc = eval_node(&step_ctx, step_id);
                ctx->steps = step_ctx.steps;
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
         * Let: binding edge = value to bind, continuation edge = body (Lambda).
         * Evaluate binding, bind result to the Lambda's binder, evaluate body.
         */
        uint64_t bind_id, cont_id;
        int have_bind = find_binding(ctx->graph, node_id, &bind_id);
        int have_cont = find_continuation(ctx->graph, node_id, &cont_id);

        if (have_bind && have_cont) {
            /* Evaluate the binding expression */
            ctx->depth++;
            iris_value_t *bound_val = eval_node(ctx, bind_id);
            ctx->depth--;

            /* The continuation should be a Lambda — get its binder and body */
            iris_node_t *cont_node = iris_graph_raw_find_node(ctx->graph, cont_id);
            if (cont_node && cont_node->kind == NK_LAMBDA) {
                uint32_t binder_id = cont_node->payload.lambda.binder_id;
                uint64_t body_id;
                if (find_continuation(ctx->graph, cont_id, &body_id)) {
                    /* Push the binding into the environment */
                    uint32_t saved_env = ctx->env_count;
                    env_push(ctx, binder_id, bound_val);

                    /* Clear memo for the body since env changed */
                    uint32_t saved_memo = ctx->memo_count;

                    ctx->depth++;
                    iris_value_t *result = eval_node(ctx, body_id);
                    ctx->depth--;

                    /* Restore env and memo */
                    ctx->env_count = saved_env;
                    ctx->memo_count = saved_memo;
                    return result;
                }
            }
            /* Non-Lambda continuation: just evaluate it */
            ctx->depth++;
            iris_value_t *result = eval_node(ctx, cont_id);
            ctx->depth--;
            return result;
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
        return iris_unit();
    }
}

/* -----------------------------------------------------------------------
 * Public entry point
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_graph(iris_graph_t *g, iris_value_t *input) {
    if (!g) return iris_unit();

    iris_value_t *self_prog = iris_program(g);

    /* Wrap single-value input in a 1-element tuple.
     * This matches the Rust evaluator convention: evaluate(graph, &[val])
     * where InputRef(0) returns inputs[0] = val (the whole value).
     * Without wrapping, a tuple input like tokens=(t1,t2,...) would be
     * decomposed by InputRef(0) → tokens[0] instead of → tokens. */
    iris_value_t *wrapped = input ? input : iris_unit();
    if (wrapped->tag != IRIS_TUPLE || wrapped->tuple.len != 1) {
        /* Not already a 1-tuple — wrap it */
        iris_value_t **elems = malloc(sizeof(iris_value_t *));
        elems[0] = wrapped;
        wrapped = iris_tuple(elems, 1);
    }

    eval_ctx_t ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.graph        = g;
    ctx.input        = wrapped;
    ctx.self_program = self_prog;
    ctx.depth        = 0;
    ctx.steps        = 0;

    return eval_node(&ctx, g->root);
}

iris_value_t *iris_eval_node(iris_graph_t *g, uint64_t node_id,
                             iris_value_t *input, uint32_t depth) {
    if (!g) return iris_unit();

    iris_value_t *self_prog = iris_program(g);

    eval_ctx_t ctx;
    memset(&ctx, 0, sizeof(ctx));
    ctx.graph        = g;
    ctx.input        = input ? input : iris_unit();
    ctx.self_program = self_prog;
    ctx.depth        = depth;
    ctx.steps        = 0;

    return eval_node(&ctx, node_id);
}
