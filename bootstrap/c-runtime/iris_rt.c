/*
 * iris_rt.c — Value constructors, memory management, accessors
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <assert.h>

#include "iris_rt.h"
#include "iris_graph.h"

/* -----------------------------------------------------------------------
 * Allocation helper
 * ----------------------------------------------------------------------- */

static iris_value_t *alloc_value(iris_type_t type) {
    iris_value_t *v = (iris_value_t *)calloc(1, sizeof(iris_value_t));
    if (!v) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    v->refcount = 1;
    v->type = type;
    return v;
}

/* -----------------------------------------------------------------------
 * Constructors
 * ----------------------------------------------------------------------- */

iris_value_t *iris_int(int64_t val) {
    iris_value_t *v = alloc_value(IRIS_INT);
    v->i = val;
    return v;
}

iris_value_t *iris_f64(double val) {
    iris_value_t *v = alloc_value(IRIS_F64);
    v->f64 = val;
    return v;
}

iris_value_t *iris_bool(int val) {
    iris_value_t *v = alloc_value(IRIS_BOOL);
    v->b = val ? 1 : 0;
    return v;
}

iris_value_t *iris_string(const char *s) {
    size_t len = s ? strlen(s) : 0;
    return iris_string_len(s, len);
}

iris_value_t *iris_string_len(const char *s, size_t len) {
    iris_value_t *v = alloc_value(IRIS_STRING);
    v->str.data = (char *)malloc(len + 1);
    if (!v->str.data) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    if (s && len > 0) memcpy(v->str.data, s, len);
    v->str.data[len] = '\0';
    v->str.len = len;
    return v;
}

iris_value_t *iris_unit(void) {
    iris_value_t *v = alloc_value(IRIS_UNIT);
    return v;
}

iris_value_t *iris_tuple(iris_value_t **elems, size_t len) {
    iris_value_t *v = alloc_value(IRIS_TUPLE);
    v->tuple.len = len;
    if (len > 0) {
        v->tuple.elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * len);
        if (!v->tuple.elems) { fprintf(stderr, "iris: out of memory\n"); abort(); }
        for (size_t i = 0; i < len; i++) {
            v->tuple.elems[i] = elems[i];
            iris_retain(elems[i]);
        }
    } else {
        v->tuple.elems = NULL;
    }
    return v;
}

iris_value_t *iris_tuple_empty(void) {
    return iris_tuple(NULL, 0);
}

iris_value_t *iris_tagged(uint16_t tag, iris_value_t *payload) {
    iris_value_t *v = alloc_value(IRIS_TAGGED);
    v->tagged.tag = tag;
    v->tagged.payload = payload;
    iris_retain(payload);
    return v;
}

iris_value_t *iris_program(iris_graph_t *g) {
    iris_value_t *v = alloc_value(IRIS_PROGRAM);
    v->graph = g;
    iris_graph_retain(g);
    return v;
}

iris_value_t *iris_bytes(const uint8_t *data, size_t len) {
    iris_value_t *v = alloc_value(IRIS_BYTES);
    v->bytes.len = len;
    if (len > 0) {
        v->bytes.data = (uint8_t *)malloc(len);
        if (!v->bytes.data) { fprintf(stderr, "iris: out of memory\n"); abort(); }
        memcpy(v->bytes.data, data, len);
    } else {
        v->bytes.data = NULL;
    }
    return v;
}

/* -----------------------------------------------------------------------
 * Reference counting
 * ----------------------------------------------------------------------- */

void iris_retain(iris_value_t *v) {
    if (v) v->refcount++;
}

void iris_release(iris_value_t *v) {
    if (!v) return;
    if (v->refcount > 1) { v->refcount--; return; }

    /* refcount == 1 -> free */
    switch (v->type) {
    case IRIS_STRING:
        free(v->str.data);
        break;
    case IRIS_TUPLE:
        for (size_t i = 0; i < v->tuple.len; i++) {
            iris_release(v->tuple.elems[i]);
        }
        free(v->tuple.elems);
        break;
    case IRIS_TAGGED:
        iris_release(v->tagged.payload);
        break;
    case IRIS_PROGRAM:
        iris_graph_release(v->graph);
        break;
    case IRIS_BYTES:
        free(v->bytes.data);
        break;
    default:
        break;
    }
    free(v);
}

/* -----------------------------------------------------------------------
 * Accessors
 * ----------------------------------------------------------------------- */

int iris_is_truthy(iris_value_t *v) {
    if (!v) return 0;
    switch (v->type) {
    case IRIS_BOOL:  return v->b;
    case IRIS_INT:   return v->i != 0;
    case IRIS_UNIT:  return 0;
    default:         return 1;
    }
}

int64_t iris_as_int(iris_value_t *v) {
    assert(v && v->type == IRIS_INT);
    return v->i;
}

double iris_as_f64(iris_value_t *v) {
    assert(v && v->type == IRIS_F64);
    return v->f64;
}

int iris_as_bool(iris_value_t *v) {
    assert(v && v->type == IRIS_BOOL);
    return v->b;
}

const char *iris_as_string(iris_value_t *v, size_t *out_len) {
    assert(v && v->type == IRIS_STRING);
    if (out_len) *out_len = v->str.len;
    return v->str.data;
}

iris_value_t *iris_tuple_get(iris_value_t *tuple, size_t idx) {
    assert(tuple && tuple->type == IRIS_TUPLE);
    if (idx >= tuple->tuple.len) return iris_unit();
    iris_value_t *elem = tuple->tuple.elems[idx];
    iris_retain(elem);
    return elem;
}

size_t iris_tuple_len(iris_value_t *v) {
    if (!v) return 0;
    if (v->type == IRIS_TUPLE) return v->tuple.len;
    return 0;
}

uint16_t iris_tagged_tag(iris_value_t *v) {
    assert(v && v->type == IRIS_TAGGED);
    return v->tagged.tag;
}

iris_value_t *iris_tagged_payload(iris_value_t *v) {
    assert(v && v->type == IRIS_TAGGED);
    iris_retain(v->tagged.payload);
    return v->tagged.payload;
}

/* -----------------------------------------------------------------------
 * Numeric coercion
 * ----------------------------------------------------------------------- */

int64_t iris_coerce_int(iris_value_t *v) {
    if (!v) return 0;
    switch (v->type) {
    case IRIS_INT:  return v->i;
    case IRIS_BOOL: return v->b ? 1 : 0;
    case IRIS_F64:  return (int64_t)v->f64;
    default:        return 0;
    }
}

double iris_coerce_f64(iris_value_t *v) {
    if (!v) return 0.0;
    switch (v->type) {
    case IRIS_F64:  return v->f64;
    case IRIS_INT:  return (double)v->i;
    case IRIS_BOOL: return v->b ? 1.0 : 0.0;
    default:        return 0.0;
    }
}

int iris_is_float_op2(iris_value_t *a, iris_value_t *b) {
    return (a && a->type == IRIS_F64) || (b && b->type == IRIS_F64);
}

/* -----------------------------------------------------------------------
 * Printing
 * ----------------------------------------------------------------------- */

void iris_fprint_value(void *fp, iris_value_t *v) {
    FILE *f = (FILE *)fp;
    if (!v) { fprintf(f, "null"); return; }
    switch (v->type) {
    case IRIS_UNIT:
        fprintf(f, "()");
        break;
    case IRIS_INT:
        fprintf(f, "%ld", (long)v->i);
        break;
    case IRIS_F64:
        fprintf(f, "%g", v->f64);
        break;
    case IRIS_BOOL:
        fprintf(f, "%s", v->b ? "true" : "false");
        break;
    case IRIS_STRING:
        fprintf(f, "\"%.*s\"", (int)v->str.len, v->str.data);
        break;
    case IRIS_TUPLE:
        fprintf(f, "(");
        for (size_t i = 0; i < v->tuple.len; i++) {
            if (i > 0) fprintf(f, ", ");
            iris_fprint_value(fp, v->tuple.elems[i]);
        }
        fprintf(f, ")");
        break;
    case IRIS_TAGGED:
        fprintf(f, "Tagged(%u, ", v->tagged.tag);
        iris_fprint_value(fp, v->tagged.payload);
        fprintf(f, ")");
        break;
    case IRIS_PROGRAM:
        fprintf(f, "<Program>");
        break;
    case IRIS_BYTES:
        fprintf(f, "<Bytes[%zu]>", v->bytes.len);
        break;
    }
}

void iris_print_value(iris_value_t *v) {
    iris_fprint_value(stdout, v);
    printf("\n");
}
