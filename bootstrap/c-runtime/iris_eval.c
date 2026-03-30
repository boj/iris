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
        return iris_unit();
    }
}

/* -----------------------------------------------------------------------
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
}
