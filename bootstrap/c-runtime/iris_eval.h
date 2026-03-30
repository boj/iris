/*
 * iris_eval.h — Tree-walking evaluator declarations
 */

#ifndef IRIS_EVAL_H
#define IRIS_EVAL_H

#include "iris_rt.h"
#include "iris_graph.h"

/* Maximum evaluation limits */
#define IRIS_MAX_DEPTH  256
#define IRIS_MAX_STEPS  500000

/* -----------------------------------------------------------------------
 * Environment: binder_id -> value mapping
 * ----------------------------------------------------------------------- */

typedef struct iris_env_entry {
    uint32_t       binder;
    iris_value_t  *value;
} iris_env_entry_t;

typedef struct iris_env {
    iris_env_entry_t *entries;
    size_t            len;
    size_t            cap;
} iris_env_t;

iris_env_t *iris_env_new(void);
void        iris_env_free(iris_env_t *env);
void        iris_env_set(iris_env_t *env, uint32_t binder, iris_value_t *val);
iris_value_t *iris_env_get(iris_env_t *env, uint32_t binder);
iris_env_t *iris_env_clone(iris_env_t *env);

/* -----------------------------------------------------------------------
 * Evaluator context
 * ----------------------------------------------------------------------- */

typedef struct iris_eval_ctx {
    iris_graph_t *graph;
    iris_env_t   *env;
    uint64_t      step_count;
    uint64_t      max_steps;
} iris_eval_ctx_t;

/* -----------------------------------------------------------------------
 * Evaluation entry points
 * ----------------------------------------------------------------------- */

/* Evaluate a graph from its root with the given inputs tuple. */
iris_value_t *iris_eval(iris_graph_t *graph, iris_value_t *inputs);

/* Evaluate a graph with an additional binder->value binding. */
iris_value_t *iris_eval_env(iris_graph_t *graph, uint32_t binder,
                            iris_value_t *value, iris_value_t *inputs);

/* Internal: evaluate a node in context. */
iris_value_t *iris_eval_node(iris_eval_ctx_t *ctx, uint64_t node_id,
                             uint32_t depth);

/* -----------------------------------------------------------------------
 * Primitive dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_dispatch_prim(iris_eval_ctx_t *ctx, uint8_t opcode,
                                 iris_value_t **args, size_t nargs);

/* -----------------------------------------------------------------------
 * Effect dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_dispatch_effect(uint8_t effect_tag,
                                   iris_value_t **args, size_t nargs);

#endif /* IRIS_EVAL_H */
