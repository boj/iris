/*
 * iris_rt.h — IRIS bootstrap C runtime: value representation & memory management
 *
 * Refcounted tagged-union values for the IRIS tree-walking evaluator.
 * Values are DAGs (no cycles), so reference counting suffices.
 */

#ifndef IRIS_RT_H
#define IRIS_RT_H

#include <stdint.h>
#include <stddef.h>

/* -----------------------------------------------------------------------
 * Value type tags
 * ----------------------------------------------------------------------- */

typedef enum {
    IRIS_UNIT = 0,
    IRIS_INT,
    IRIS_F64,
    IRIS_BOOL,
    IRIS_STRING,
    IRIS_TUPLE,
    IRIS_TAGGED,
    IRIS_PROGRAM,
    IRIS_BYTES,
} iris_type_t;

/* Forward declarations */
typedef struct iris_value  iris_value_t;
typedef struct iris_graph  iris_graph_t;

/* -----------------------------------------------------------------------
 * Value: refcounted tagged union
 * ----------------------------------------------------------------------- */

struct iris_value {
    uint32_t   refcount;
    iris_type_t type;
    union {
        int64_t  i;                                          /* INT    */
        double   f64;                                        /* F64    */
        int      b;                                          /* BOOL   */
        struct { char    *data; size_t len; } str;           /* STRING */
        struct { iris_value_t **elems; size_t len; } tuple;  /* TUPLE  */
        struct { uint16_t tag; iris_value_t *payload; } tagged; /* TAGGED */
        iris_graph_t *graph;                                 /* PROGRAM */
        struct { uint8_t *data; size_t len; } bytes;         /* BYTES  */
    };
};

/* -----------------------------------------------------------------------
 * Value constructors
 * ----------------------------------------------------------------------- */

iris_value_t *iris_int(int64_t v);
iris_value_t *iris_f64(double v);
iris_value_t *iris_bool(int v);
iris_value_t *iris_string(const char *s);
iris_value_t *iris_string_len(const char *s, size_t len);
iris_value_t *iris_unit(void);
iris_value_t *iris_tuple(iris_value_t **elems, size_t len);
iris_value_t *iris_tuple_empty(void);
iris_value_t *iris_tagged(uint16_t tag, iris_value_t *payload);
iris_value_t *iris_program(iris_graph_t *g);
iris_value_t *iris_bytes(const uint8_t *data, size_t len);

/* -----------------------------------------------------------------------
 * Reference counting
 * ----------------------------------------------------------------------- */

void iris_retain(iris_value_t *v);
void iris_release(iris_value_t *v);

/* -----------------------------------------------------------------------
 * Value accessors
 * ----------------------------------------------------------------------- */

int      iris_is_truthy(iris_value_t *v);
int64_t  iris_as_int(iris_value_t *v);
double   iris_as_f64(iris_value_t *v);
int      iris_as_bool(iris_value_t *v);
const char *iris_as_string(iris_value_t *v, size_t *out_len);

iris_value_t *iris_tuple_get(iris_value_t *tuple, size_t idx);
size_t        iris_tuple_len(iris_value_t *v);

uint16_t      iris_tagged_tag(iris_value_t *v);
iris_value_t *iris_tagged_payload(iris_value_t *v);

/* -----------------------------------------------------------------------
 * Value printing
 * ----------------------------------------------------------------------- */

void iris_print_value(iris_value_t *v);
void iris_fprint_value(void *fp, iris_value_t *v);

/* -----------------------------------------------------------------------
 * Numeric coercion helpers
 * ----------------------------------------------------------------------- */

int64_t iris_coerce_int(iris_value_t *v);
double  iris_coerce_f64(iris_value_t *v);
int     iris_is_float_op2(iris_value_t *a, iris_value_t *b);

/* -----------------------------------------------------------------------
 * Primitive operations (iris_prims.c)
 * ----------------------------------------------------------------------- */

/* Arithmetic */
iris_value_t *iris_add(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_sub(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_mul(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_div(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_mod(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_neg(iris_value_t *a);
iris_value_t *iris_abs(iris_value_t *a);
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

/* Logic/Bitwise */
iris_value_t *iris_and(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_or(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_not(iris_value_t *a);

/* List/Tuple */
iris_value_t *iris_list_len(iris_value_t *v);
iris_value_t *iris_list_nth(iris_value_t *list, iris_value_t *idx);
iris_value_t *iris_list_append(iris_value_t *list, iris_value_t *elem);
iris_value_t *iris_list_head(iris_value_t *list);
iris_value_t *iris_list_tail(iris_value_t *list);
iris_value_t *iris_list_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_list_reverse(iris_value_t *list);

/* String */
iris_value_t *iris_str_len(iris_value_t *s);
iris_value_t *iris_str_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_str_slice(iris_value_t *s, iris_value_t *start, iris_value_t *end);
iris_value_t *iris_str_to_int(iris_value_t *s);
iris_value_t *iris_int_to_string(iris_value_t *v);

/* Graph introspection */
iris_value_t *iris_graph_get_root(iris_value_t *prog);
iris_value_t *iris_graph_get_kind(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_get_prim_op(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_outgoing(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_set_root(iris_value_t *prog, iris_value_t *node_id);
iris_value_t *iris_graph_eval(iris_value_t *prog, iris_value_t *inputs);
iris_value_t *iris_graph_eval_env(iris_value_t *prog, iris_value_t *binder, iris_value_t *val, iris_value_t *inputs);
iris_value_t *iris_graph_edge_target(iris_value_t *prog, iris_value_t *src, iris_value_t *port, iris_value_t *label);
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

/* Compiled interpreter entry point (generated) */
iris_value_t *iris_interpret(iris_value_t *program, iris_value_t *inputs);

#endif /* IRIS_RT_H */
