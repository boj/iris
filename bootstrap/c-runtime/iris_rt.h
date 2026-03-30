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

#endif /* IRIS_RT_H */
