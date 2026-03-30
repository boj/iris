/*
 * iris_rt.h -- Value-level runtime types for the IRIS C bootstrap.
 *
 * All functions that the compiled interpreter calls operate on iris_value_t*.
 * The graph-level structs (iris_graph_t, iris_node_t) are internal to
 * iris_graph.h and are accessed through value-level wrappers.
 */
#ifndef IRIS_RT_H
#define IRIS_RT_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

/* Forward declarations */
typedef struct iris_graph iris_graph_t;
typedef struct iris_node  iris_node_t;

/* -----------------------------------------------------------------------
 * Value type tags
 * ----------------------------------------------------------------------- */

typedef enum {
    IRIS_INT      = 0,
    IRIS_FLOAT64  = 1,
    IRIS_BOOL     = 2,
    IRIS_UNIT     = 3,
    IRIS_TUPLE    = 4,
    IRIS_PROGRAM  = 5,
    IRIS_TAGGED   = 6,
    IRIS_BYTES    = 7,
    IRIS_STRING   = 8,
} iris_value_tag_t;

/* -----------------------------------------------------------------------
 * Value
 * ----------------------------------------------------------------------- */

typedef struct iris_value {
    iris_value_tag_t tag;
    union {
        int64_t  i;
        double   f;
        bool     b;
        /* IRIS_TUPLE: elements stored in a separate allocation */
        struct {
            struct iris_value **elems;
            uint32_t            len;
        } tuple;
        /* IRIS_PROGRAM: pointer to an iris_graph_t */
        iris_graph_t *graph;
        /* IRIS_TAGGED: tag + payload */
        struct {
            uint16_t             tag_index;
            struct iris_value   *payload;
        } tagged;
        /* IRIS_BYTES */
        struct {
            uint8_t  *data;
            uint32_t  len;
        } bytes;
        /* IRIS_STRING */
        struct {
            char     *data;
            uint32_t  len;
        } str;
    };
} iris_value_t;

/* -----------------------------------------------------------------------
 * Value constructors (defined in iris_prims.c)
 * ----------------------------------------------------------------------- */

iris_value_t *iris_int(int64_t n);
iris_value_t *iris_float64(double f);
iris_value_t *iris_bool(bool b);
iris_value_t *iris_unit(void);
iris_value_t *iris_tuple(iris_value_t **elems, uint32_t len);
iris_value_t *iris_program(iris_graph_t *g);
iris_value_t *iris_tagged(uint16_t tag, iris_value_t *payload);
iris_value_t *iris_string(const char *data, uint32_t len);

/* Extract raw values */
static inline int64_t iris_as_int(iris_value_t *v) {
    if (!v) return 0;
    if (v->tag == IRIS_INT)  return v->i;
    if (v->tag == IRIS_BOOL) return v->b ? 1 : 0;
    return 0;
}

static inline double iris_as_float(iris_value_t *v) {
    if (!v) return 0.0;
    if (v->tag == IRIS_FLOAT64) return v->f;
    if (v->tag == IRIS_INT)     return (double)v->i;
    return 0.0;
}

static inline bool iris_as_bool(iris_value_t *v) {
    if (!v) return false;
    if (v->tag == IRIS_BOOL) return v->b;
    if (v->tag == IRIS_INT)  return v->i != 0;
    return false;
}

/* -----------------------------------------------------------------------
 * Value-level graph introspection (called by compiled interpreter)
 * Implemented in iris_prims.c.
 * ----------------------------------------------------------------------- */

iris_value_t *iris_graph_get_root(iris_value_t *prog);
iris_value_t *iris_graph_get_kind(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_prim_op(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_outgoing(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_set_root(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_eval(iris_value_t *prog, iris_value_t *inputs);
iris_value_t *iris_graph_eval_env(iris_value_t *prog, iris_value_t *binder,
                                  iris_value_t *value, iris_value_t *inputs);
iris_value_t *iris_graph_edge_target(iris_value_t *prog, iris_value_t *src,
                                     iris_value_t *port, iris_value_t *label);
iris_value_t *iris_graph_get_binder(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_tag(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_field_index(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_effect_tag(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_lit_type_tag(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_lit_value(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_value_get_tag(iris_value_t *v);
iris_value_t *iris_value_get_payload(iris_value_t *v);
iris_value_t *iris_value_make_tagged(iris_value_t *tag, iris_value_t *payload);
iris_value_t *iris_perform_effect(iris_value_t *tag, iris_value_t *args);
iris_value_t *iris_list_len(iris_value_t *v);
iris_value_t *iris_list_nth(iris_value_t *list, iris_value_t *idx);

/* -----------------------------------------------------------------------
 * Truthiness + named prim wrappers (called by compiled interpreter)
 * ----------------------------------------------------------------------- */

static inline bool iris_is_truthy(iris_value_t *v) {
    if (!v) return false;
    if (v->tag == IRIS_INT)  return v->i != 0;
    if (v->tag == IRIS_BOOL) return v->b;
    return true; /* non-null, non-zero = truthy */
}

/* Arithmetic */
iris_value_t *iris_add(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_sub(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_mul(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_div(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_mod(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_neg(iris_value_t *a);
iris_value_t *iris_abs_val(iris_value_t *a);
iris_value_t *iris_min(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_max(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_pow(iris_value_t *a, iris_value_t *b);

/* Comparison */
iris_value_t *iris_eq(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_ne(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_lt(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_gt(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_le(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_ge(iris_value_t *a, iris_value_t *b);

/* Logic */
iris_value_t *iris_and(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_or(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_not(iris_value_t *a);

/* String */
iris_value_t *iris_str_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_str_len(iris_value_t *s);
iris_value_t *iris_str_to_int(iris_value_t *s);
iris_value_t *iris_int_to_string(iris_value_t *v);
iris_value_t *iris_list_append(iris_value_t *list, iris_value_t *elem);
iris_value_t *iris_tuple_get(iris_value_t *tup, iris_value_t *idx);
iris_value_t *iris_tuple_len_val(iris_value_t *v);

/* Compiled interpreter entry point (generated) */
iris_value_t *iris_interpret(iris_value_t *program, iris_value_t *inputs);

/* -----------------------------------------------------------------------
 * Prim dispatch (called by evaluator)
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_prim(uint8_t opcode, iris_value_t **args, uint32_t nargs,
                             iris_value_t *self_program);

/* -----------------------------------------------------------------------
 * Tree-walking evaluator (iris_eval.c)
 * ----------------------------------------------------------------------- */

iris_value_t *iris_eval_graph(iris_graph_t *g, iris_value_t *input);
iris_value_t *iris_eval_node(iris_graph_t *g, uint64_t node_id,
                             iris_value_t *input, uint32_t depth);

/* -----------------------------------------------------------------------
 * JSON graph loader (iris_graph.c)
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_load_json(const char *path);

#endif /* IRIS_RT_H */
