/*
 * iris_prims.c — Primitive opcode implementations
 *
 * Implements the ~60 opcodes dispatched by the evaluator. Grouped by:
 *   Arithmetic, Comparison, Logic/Bitwise, Conversion, String, List/Tuple,
 *   Graph introspection, Graph mutation, Math, Map/State, Misc.
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <math.h>
#include <time.h>
#include <assert.h>

#include "iris_eval.h"
#include "iris_json.h"

/* -----------------------------------------------------------------------
 * Helper macros
 * ----------------------------------------------------------------------- */

#define NEED_ARGS(n, name) do { \
    if (nargs < (n)) { \
        fprintf(stderr, "prim %s: expected %d args, got %zu\n", name, (n), nargs); \
        return iris_unit(); \
    } \
} while(0)

#define ARG(i) (args[i])

/* -----------------------------------------------------------------------
 * Arithmetic binop: handles Int+Int, Float+Float, mixed
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_arith_binop(
    iris_value_t *a, iris_value_t *b,
    int64_t (*int_op)(int64_t, int64_t),
    double (*float_op)(double, double))
{
    if (iris_is_float_op2(a, b)) {
        double fa = iris_coerce_f64(a);
        double fb = iris_coerce_f64(b);
        return iris_f64(float_op(fa, fb));
    }
    return iris_int(int_op(iris_coerce_int(a), iris_coerce_int(b)));
}

static int64_t op_add(int64_t a, int64_t b) { return a + b; }
static int64_t op_sub(int64_t a, int64_t b) { return a - b; }
static int64_t op_mul(int64_t a, int64_t b) { return a * b; }
static int64_t op_min(int64_t a, int64_t b) { return a < b ? a : b; }
static int64_t op_max(int64_t a, int64_t b) { return a > b ? a : b; }

static double fop_add(double a, double b) { return a + b; }
static double fop_sub(double a, double b) { return a - b; }
static double fop_mul(double a, double b) { return a * b; }
static double fop_min(double a, double b) { return a < b ? a : b; }
static double fop_max(double a, double b) { return a > b ? a : b; }

/* -----------------------------------------------------------------------
 * Comparison
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_cmp(iris_value_t *a, iris_value_t *b,
                              int (*test)(int)) {
    if (iris_is_float_op2(a, b)) {
        double fa = iris_coerce_f64(a);
        double fb = iris_coerce_f64(b);
        int cmp = (fa > fb) - (fa < fb);
        return iris_bool(test(cmp));
    }
    int64_t ia = iris_coerce_int(a);
    int64_t ib = iris_coerce_int(b);
    int cmp = (ia > ib) - (ia < ib);
    return iris_bool(test(cmp));
}

static int cmp_eq(int c) { return c == 0; }
static int cmp_ne(int c) { return c != 0; }
static int cmp_lt(int c) { return c < 0; }
static int cmp_gt(int c) { return c > 0; }
static int cmp_le(int c) { return c <= 0; }
static int cmp_ge(int c) { return c >= 0; }

/* -----------------------------------------------------------------------
 * String primitives
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_str_len(iris_value_t *v) {
    if (v->type == IRIS_STRING) return iris_int((int64_t)v->str.len);
    return iris_int(0);
}

static iris_value_t *prim_str_concat(iris_value_t *a, iris_value_t *b) {
    if (a->type != IRIS_STRING || b->type != IRIS_STRING) return iris_string("");
    size_t total = a->str.len + b->str.len;
    char *buf = (char *)malloc(total + 1);
    memcpy(buf, a->str.data, a->str.len);
    memcpy(buf + a->str.len, b->str.data, b->str.len);
    buf[total] = '\0';
    iris_value_t *result = iris_string_len(buf, total);
    free(buf);
    return result;
}

static iris_value_t *prim_str_slice(iris_value_t **args, size_t nargs) {
    NEED_ARGS(3, "str_slice");
    if (ARG(0)->type != IRIS_STRING) return iris_string("");
    int64_t start = iris_coerce_int(ARG(1));
    int64_t end = iris_coerce_int(ARG(2));
    size_t len = ARG(0)->str.len;
    if (start < 0) start = 0;
    if ((size_t)end > len) end = (int64_t)len;
    if (start >= end) return iris_string("");
    return iris_string_len(ARG(0)->str.data + start, (size_t)(end - start));
}

static iris_value_t *prim_str_contains(iris_value_t *hay, iris_value_t *needle) {
    if (hay->type != IRIS_STRING || needle->type != IRIS_STRING)
        return iris_bool(0);
    return iris_bool(strstr(hay->str.data, needle->str.data) != NULL);
}

static iris_value_t *prim_str_split(iris_value_t *s, iris_value_t *delim) {
    if (s->type != IRIS_STRING || delim->type != IRIS_STRING)
        return iris_tuple_empty();

    /* Split string by delimiter */
    size_t cap = 8, count = 0;
    iris_value_t **parts = (iris_value_t **)malloc(sizeof(iris_value_t *) * cap);
    const char *p = s->str.data;
    size_t dlen = delim->str.len;

    if (dlen == 0) {
        /* Split into characters */
        for (size_t i = 0; i < s->str.len; i++) {
            if (count >= cap) { cap *= 2; parts = (iris_value_t **)realloc(parts, sizeof(iris_value_t *) * cap); }
            parts[count++] = iris_string_len(s->str.data + i, 1);
        }
    } else {
        while (*p) {
            const char *found = strstr(p, delim->str.data);
            if (!found) {
                if (count >= cap) { cap *= 2; parts = (iris_value_t **)realloc(parts, sizeof(iris_value_t *) * cap); }
                parts[count++] = iris_string(p);
                break;
            }
            if (count >= cap) { cap *= 2; parts = (iris_value_t **)realloc(parts, sizeof(iris_value_t *) * cap); }
            parts[count++] = iris_string_len(p, (size_t)(found - p));
            p = found + dlen;
        }
    }

    iris_value_t *result = iris_tuple(parts, count);
    for (size_t i = 0; i < count; i++) iris_release(parts[i]);
    free(parts);
    return result;
}

static iris_value_t *prim_str_join(iris_value_t *list, iris_value_t *delim) {
    if (list->type != IRIS_TUPLE || delim->type != IRIS_STRING)
        return iris_string("");

    /* Calculate total length */
    size_t total = 0;
    for (size_t i = 0; i < list->tuple.len; i++) {
        iris_value_t *elem = list->tuple.elems[i];
        if (elem->type == IRIS_STRING) total += elem->str.len;
        if (i > 0) total += delim->str.len;
    }

    char *buf = (char *)malloc(total + 1);
    size_t pos = 0;
    for (size_t i = 0; i < list->tuple.len; i++) {
        if (i > 0 && delim->str.len > 0) {
            memcpy(buf + pos, delim->str.data, delim->str.len);
            pos += delim->str.len;
        }
        iris_value_t *elem = list->tuple.elems[i];
        if (elem->type == IRIS_STRING) {
            memcpy(buf + pos, elem->str.data, elem->str.len);
            pos += elem->str.len;
        }
    }
    buf[pos] = '\0';

    iris_value_t *result = iris_string_len(buf, pos);
    free(buf);
    return result;
}

static iris_value_t *prim_str_to_int(iris_value_t *v) {
    if (v->type != IRIS_STRING) return iris_int(0);
    return iris_int(strtoll(v->str.data, NULL, 10));
}

static iris_value_t *prim_int_to_string(iris_value_t *v) {
    char buf[32];
    snprintf(buf, sizeof(buf), "%ld", (long)iris_coerce_int(v));
    return iris_string(buf);
}

static iris_value_t *prim_str_eq(iris_value_t *a, iris_value_t *b) {
    if (a->type != IRIS_STRING || b->type != IRIS_STRING)
        return iris_bool(0);
    return iris_bool(a->str.len == b->str.len &&
                     memcmp(a->str.data, b->str.data, a->str.len) == 0);
}

static iris_value_t *prim_str_starts_with(iris_value_t *s, iris_value_t *prefix) {
    if (s->type != IRIS_STRING || prefix->type != IRIS_STRING) return iris_bool(0);
    if (prefix->str.len > s->str.len) return iris_bool(0);
    return iris_bool(memcmp(s->str.data, prefix->str.data, prefix->str.len) == 0);
}

static iris_value_t *prim_str_ends_with(iris_value_t *s, iris_value_t *suffix) {
    if (s->type != IRIS_STRING || suffix->type != IRIS_STRING) return iris_bool(0);
    if (suffix->str.len > s->str.len) return iris_bool(0);
    size_t offset = s->str.len - suffix->str.len;
    return iris_bool(memcmp(s->str.data + offset, suffix->str.data, suffix->str.len) == 0);
}

static iris_value_t *prim_str_replace(iris_value_t **args, size_t nargs) {
    NEED_ARGS(3, "str_replace");
    if (ARG(0)->type != IRIS_STRING || ARG(1)->type != IRIS_STRING ||
        ARG(2)->type != IRIS_STRING)
        return iris_string("");

    const char *src = ARG(0)->str.data;
    size_t slen = ARG(0)->str.len;
    const char *pat = ARG(1)->str.data;
    size_t plen = ARG(1)->str.len;
    const char *rep = ARG(2)->str.data;
    size_t rlen = ARG(2)->str.len;

    if (plen == 0) { iris_retain(ARG(0)); return ARG(0); }

    /* Build result */
    size_t cap = slen + 64;
    char *buf = (char *)malloc(cap);
    size_t pos = 0;
    const char *p = src;

    while (*p) {
        const char *found = strstr(p, pat);
        if (!found) {
            size_t rem = slen - (size_t)(p - src);
            while (pos + rem >= cap) { cap *= 2; buf = (char *)realloc(buf, cap); }
            memcpy(buf + pos, p, rem);
            pos += rem;
            break;
        }
        size_t before = (size_t)(found - p);
        while (pos + before + rlen >= cap) { cap *= 2; buf = (char *)realloc(buf, cap); }
        memcpy(buf + pos, p, before);
        pos += before;
        memcpy(buf + pos, rep, rlen);
        pos += rlen;
        p = found + plen;
    }
    buf[pos] = '\0';

    iris_value_t *result = iris_string_len(buf, pos);
    free(buf);
    return result;
}

static iris_value_t *prim_str_trim(iris_value_t *v) {
    if (v->type != IRIS_STRING) return iris_string("");
    const char *s = v->str.data;
    size_t len = v->str.len;
    size_t start = 0, end = len;
    while (start < end && (s[start] == ' ' || s[start] == '\t' ||
           s[start] == '\n' || s[start] == '\r')) start++;
    while (end > start && (s[end-1] == ' ' || s[end-1] == '\t' ||
           s[end-1] == '\n' || s[end-1] == '\r')) end--;
    return iris_string_len(s + start, end - start);
}

static iris_value_t *prim_str_upper(iris_value_t *v) {
    if (v->type != IRIS_STRING) return iris_string("");
    char *buf = (char *)malloc(v->str.len + 1);
    for (size_t i = 0; i < v->str.len; i++) {
        char c = v->str.data[i];
        buf[i] = (c >= 'a' && c <= 'z') ? (char)(c - 32) : c;
    }
    buf[v->str.len] = '\0';
    iris_value_t *r = iris_string_len(buf, v->str.len);
    free(buf);
    return r;
}

static iris_value_t *prim_str_lower(iris_value_t *v) {
    if (v->type != IRIS_STRING) return iris_string("");
    char *buf = (char *)malloc(v->str.len + 1);
    for (size_t i = 0; i < v->str.len; i++) {
        char c = v->str.data[i];
        buf[i] = (c >= 'A' && c <= 'Z') ? (char)(c + 32) : c;
    }
    buf[v->str.len] = '\0';
    iris_value_t *r = iris_string_len(buf, v->str.len);
    free(buf);
    return r;
}

static iris_value_t *prim_str_chars(iris_value_t *v) {
    if (v->type != IRIS_STRING) return iris_tuple_empty();
    size_t n = v->str.len;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) {
        elems[i] = iris_string_len(v->str.data + i, 1);
    }
    iris_value_t *r = iris_tuple(elems, n);
    for (size_t i = 0; i < n; i++) iris_release(elems[i]);
    free(elems);
    return r;
}

static iris_value_t *prim_char_at(iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "char_at");
    if (ARG(0)->type != IRIS_STRING) return iris_string("");
    int64_t idx = iris_coerce_int(ARG(1));
    if (idx < 0 || (size_t)idx >= ARG(0)->str.len) return iris_string("");
    return iris_string_len(ARG(0)->str.data + idx, 1);
}

/* -----------------------------------------------------------------------
 * List/tuple primitives
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_list_append(iris_value_t *list, iris_value_t *elem) {
    size_t old_len = (list->type == IRIS_TUPLE) ? list->tuple.len : 0;
    size_t new_len = old_len + 1;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * new_len);
    for (size_t i = 0; i < old_len; i++) {
        elems[i] = list->tuple.elems[i];
    }
    elems[old_len] = elem;
    iris_value_t *r = iris_tuple(elems, new_len);
    free(elems);
    return r;
}

static iris_value_t *prim_list_nth(iris_value_t *list, iris_value_t *idx_v) {
    if (list->type != IRIS_TUPLE) return iris_unit();
    int64_t idx = iris_coerce_int(idx_v);
    if (idx < 0 || (size_t)idx >= list->tuple.len) return iris_unit();
    iris_value_t *elem = list->tuple.elems[idx];
    iris_retain(elem);
    return elem;
}

static iris_value_t *prim_list_take(iris_value_t *list, iris_value_t *n_v) {
    if (list->type != IRIS_TUPLE) return iris_tuple_empty();
    int64_t n = iris_coerce_int(n_v);
    if (n <= 0) return iris_tuple_empty();
    if ((size_t)n > list->tuple.len) n = (int64_t)list->tuple.len;
    return iris_tuple(list->tuple.elems, (size_t)n);
}

static iris_value_t *prim_list_drop(iris_value_t *list, iris_value_t *n_v) {
    if (list->type != IRIS_TUPLE) return iris_tuple_empty();
    int64_t n = iris_coerce_int(n_v);
    if (n <= 0) { iris_retain(list); return list; }
    if ((size_t)n >= list->tuple.len) return iris_tuple_empty();
    return iris_tuple(list->tuple.elems + n, list->tuple.len - (size_t)n);
}

static int val_cmp_int(const void *a, const void *b) {
    iris_value_t *va = *(iris_value_t **)a;
    iris_value_t *vb = *(iris_value_t **)b;
    int64_t ia = iris_coerce_int(va);
    int64_t ib = iris_coerce_int(vb);
    return (ia > ib) - (ia < ib);
}

static iris_value_t *prim_list_sort(iris_value_t *list) {
    if (list->type != IRIS_TUPLE || list->tuple.len <= 1) {
        iris_retain(list); return list;
    }
    size_t n = list->tuple.len;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    memcpy(elems, list->tuple.elems, sizeof(iris_value_t *) * n);
    qsort(elems, n, sizeof(iris_value_t *), val_cmp_int);
    iris_value_t *r = iris_tuple(elems, n);
    free(elems);
    return r;
}

static iris_value_t *prim_list_dedup(iris_value_t *list) {
    if (list->type != IRIS_TUPLE || list->tuple.len <= 1) {
        iris_retain(list); return list;
    }
    size_t n = list->tuple.len;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    size_t out = 0;
    for (size_t i = 0; i < n; i++) {
        int dup = 0;
        for (size_t j = 0; j < out; j++) {
            if (iris_coerce_int(list->tuple.elems[i]) == iris_coerce_int(elems[j])) {
                dup = 1; break;
            }
        }
        if (!dup) elems[out++] = list->tuple.elems[i];
    }
    iris_value_t *r = iris_tuple(elems, out);
    free(elems);
    return r;
}

static iris_value_t *prim_list_range(iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "list_range");
    int64_t start = iris_coerce_int(ARG(0));
    int64_t end = iris_coerce_int(ARG(1));
    if (end <= start) return iris_tuple_empty();
    size_t n = (size_t)(end - start);
    if (n > 100000) n = 100000; /* safety limit */
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) {
        elems[i] = iris_int(start + (int64_t)i);
    }
    iris_value_t *r = iris_tuple(elems, n);
    for (size_t i = 0; i < n; i++) iris_release(elems[i]);
    free(elems);
    return r;
}

static iris_value_t *prim_list_concat(iris_value_t *a, iris_value_t *b) {
    size_t alen = (a->type == IRIS_TUPLE) ? a->tuple.len : 0;
    size_t blen = (b->type == IRIS_TUPLE) ? b->tuple.len : 0;
    size_t total = alen + blen;
    if (total == 0) return iris_tuple_empty();
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * total);
    for (size_t i = 0; i < alen; i++) elems[i] = a->tuple.elems[i];
    for (size_t i = 0; i < blen; i++) elems[alen + i] = b->tuple.elems[i];
    iris_value_t *r = iris_tuple(elems, total);
    free(elems);
    return r;
}

static iris_value_t *prim_list_reverse(iris_value_t *list) {
    if (list->type != IRIS_TUPLE) { iris_retain(list); return list; }
    size_t n = list->tuple.len;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) elems[i] = list->tuple.elems[n - 1 - i];
    iris_value_t *r = iris_tuple(elems, n);
    free(elems);
    return r;
}

static iris_value_t *prim_list_len(iris_value_t *v) {
    if (v->type == IRIS_TUPLE) return iris_int((int64_t)v->tuple.len);
    return iris_int(0);
}

static iris_value_t *prim_tuple_get(iris_value_t *tuple, iris_value_t *idx_v) {
    if (tuple->type != IRIS_TUPLE) return iris_unit();
    int64_t idx = iris_coerce_int(idx_v);
    if (idx < 0 || (size_t)idx >= tuple->tuple.len) return iris_unit();
    iris_value_t *elem = tuple->tuple.elems[idx];
    iris_retain(elem);
    return elem;
}

static iris_value_t *prim_tuple_len(iris_value_t *v) {
    return iris_int((int64_t)iris_tuple_len(v));
}

static iris_value_t *prim_zip(iris_value_t *a, iris_value_t *b) {
    if (a->type != IRIS_TUPLE || b->type != IRIS_TUPLE) return iris_tuple_empty();
    size_t n = a->tuple.len < b->tuple.len ? a->tuple.len : b->tuple.len;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) {
        iris_value_t *pair[2] = { a->tuple.elems[i], b->tuple.elems[i] };
        elems[i] = iris_tuple(pair, 2);
    }
    iris_value_t *r = iris_tuple(elems, n);
    for (size_t i = 0; i < n; i++) iris_release(elems[i]);
    free(elems);
    return r;
}

/* -----------------------------------------------------------------------
 * Graph introspection primitives
 * ----------------------------------------------------------------------- */

static iris_graph_t *extract_graph(iris_value_t *v) {
    if (v->type == IRIS_PROGRAM) return v->graph;
    /* Also accept tuple with program as first element */
    if (v->type == IRIS_TUPLE && v->tuple.len > 0 &&
        v->tuple.elems[0]->type == IRIS_PROGRAM) {
        return v->tuple.elems[0]->graph;
    }
    return NULL;
}

static iris_value_t *prim_graph_get_root(iris_value_t *prog) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    return iris_int((int64_t)g->root);
}

static iris_value_t *prim_graph_get_kind(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(-1);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node) return iris_int(-1);
    return iris_int((int64_t)node->kind);
}

static iris_value_t *prim_graph_get_prim_op(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(-1);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_PRIM) return iris_int(-1);
    return iris_int((int64_t)node->payload.prim_opcode);
}

static iris_value_t *prim_graph_outgoing(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_tuple_empty();
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);

    iris_edge_t *edges = NULL;
    size_t count = iris_graph_find_edges(g, nid, &edges);
    if (count == 0) return iris_tuple_empty();

    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * count);
    for (size_t i = 0; i < count; i++) {
        /* Each edge as (target, port, label) */
        iris_value_t *parts[3] = {
            iris_int((int64_t)edges[i].target),
            iris_int((int64_t)edges[i].port),
            iris_int((int64_t)edges[i].label),
        };
        elems[i] = iris_tuple(parts, 3);
        iris_release(parts[0]);
        iris_release(parts[1]);
        iris_release(parts[2]);
    }
    free(edges);

    iris_value_t *r = iris_tuple(elems, count);
    for (size_t i = 0; i < count; i++) iris_release(elems[i]);
    free(elems);
    return r;
}

static iris_value_t *prim_graph_edge_target_op(iris_value_t **args, size_t nargs) {
    NEED_ARGS(4, "graph_edge_target");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) return iris_int(0);
    uint64_t source = (uint64_t)iris_coerce_int(ARG(1));
    uint8_t port = (uint8_t)iris_coerce_int(ARG(2));
    uint8_t label = (uint8_t)iris_coerce_int(ARG(3));
    uint64_t target = iris_graph_edge_target(g, source, port, label);
    return iris_int((int64_t)target);
}

static iris_value_t *prim_graph_get_binder(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node) return iris_int(0);
    if (node->kind == NK_LAMBDA) return iris_int((int64_t)node->payload.lambda.binder_id);
    if (node->kind == NK_LETREC) return iris_int((int64_t)node->payload.letrec.binder_id);
    return iris_int(0);
}

static iris_value_t *prim_graph_get_tag(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_INJECT) return iris_int(0);
    return iris_int((int64_t)node->payload.inject.tag_index);
}

static iris_value_t *prim_graph_get_field_index(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_PROJECT) return iris_int(0);
    return iris_int((int64_t)node->payload.project.field_index);
}

static iris_value_t *prim_graph_get_effect_tag(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_EFFECT) return iris_int(0);
    return iris_int((int64_t)node->payload.effect_tag);
}

static iris_value_t *prim_graph_get_lit_type_tag(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_LIT) return iris_int(0);
    return iris_int((int64_t)node->payload.lit.type_tag);
}

static iris_value_t *prim_graph_get_lit_value(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_unit();
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node || node->kind != NK_LIT) return iris_unit();

    /* Return the literal value by interpreting its type_tag */
    uint8_t tt = node->payload.lit.type_tag;
    uint8_t *val = node->payload.lit.value;
    size_t vlen = node->payload.lit.value_len;

    switch (tt) {
    case 0x00: if (vlen >= 8) { int64_t n; memcpy(&n, val, 8); return iris_int(n); } break;
    case 0x02: if (vlen >= 8) { double f; memcpy(&f, val, 8); return iris_f64(f); } break;
    case 0x04: if (vlen >= 1) return iris_bool(val[0] != 0); break;
    case 0x06: return iris_unit();
    case 0x07: return iris_string_len((const char *)val, vlen);
    case 0xFF: if (vlen > 0) return iris_int((int64_t)val[0]); break;
    default: break;
    }
    return iris_unit();
}

static iris_value_t *prim_graph_nodes(iris_value_t *prog) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_tuple_empty();
    size_t n = g->data->node_count;
    iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
    for (size_t i = 0; i < n; i++) {
        elems[i] = iris_int((int64_t)g->data->nodes[i].id);
    }
    iris_value_t *r = iris_tuple(elems, n);
    for (size_t i = 0; i < n; i++) iris_release(elems[i]);
    free(elems);
    return r;
}

static iris_value_t *prim_graph_edge_count_op(iris_value_t *prog) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    return iris_int((int64_t)g->data->edge_count);
}

static iris_value_t *prim_graph_get_arity(iris_value_t *prog, iris_value_t *nid_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_int(0);
    uint64_t nid = (uint64_t)iris_coerce_int(nid_v);
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (!node) return iris_int(0);
    return iris_int((int64_t)node->arity);
}

/* -----------------------------------------------------------------------
 * Graph eval (evaluate a sub-program)
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_graph_eval(iris_eval_ctx_t *ctx,
                                     iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "graph_eval");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) return iris_unit();
    return iris_eval(g, ARG(1));
}

static iris_value_t *prim_graph_eval_env(iris_eval_ctx_t *ctx,
                                         iris_value_t **args, size_t nargs) {
    NEED_ARGS(4, "graph_eval_env");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) return iris_unit();
    uint32_t binder = (uint32_t)iris_coerce_int(ARG(1));
    return iris_eval_env(g, binder, ARG(2), ARG(3));
}

/* -----------------------------------------------------------------------
 * Graph set_root
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_graph_set_root_op(iris_value_t *prog,
                                             iris_value_t *new_root_v) {
    iris_graph_t *g = extract_graph(prog);
    if (!g) return iris_unit();
    uint64_t new_root = (uint64_t)iris_coerce_int(new_root_v);
    iris_graph_t *new_g = iris_graph_set_root(g, new_root);
    iris_value_t *r = iris_program(new_g);
    iris_graph_release(new_g);
    return r;
}

/* -----------------------------------------------------------------------
 * Value tag/payload ops
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_value_get_tag(iris_value_t *v) {
    if (v->type == IRIS_TAGGED) return iris_int((int64_t)v->tagged.tag);
    return iris_int(-1);
}

static iris_value_t *prim_value_get_payload(iris_value_t *v) {
    if (v->type == IRIS_TAGGED) {
        iris_retain(v->tagged.payload);
        return v->tagged.payload;
    }
    return iris_unit();
}

static iris_value_t *prim_value_make_tagged(iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "value_make_tagged");
    uint16_t tag = (uint16_t)iris_coerce_int(ARG(0));
    return iris_tagged(tag, ARG(1));
}

/* -----------------------------------------------------------------------
 * Bytes primitives
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_bytes_from_ints(iris_value_t *list) {
    if (list->type != IRIS_TUPLE) return iris_bytes(NULL, 0);
    size_t n = list->tuple.len;
    uint8_t *data = (uint8_t *)malloc(n);
    for (size_t i = 0; i < n; i++) {
        data[i] = (uint8_t)iris_coerce_int(list->tuple.elems[i]);
    }
    iris_value_t *r = iris_bytes(data, n);
    free(data);
    return r;
}

static iris_value_t *prim_bytes_concat(iris_value_t *a, iris_value_t *b) {
    if (a->type != IRIS_BYTES || b->type != IRIS_BYTES) return iris_bytes(NULL, 0);
    size_t total = a->bytes.len + b->bytes.len;
    uint8_t *data = (uint8_t *)malloc(total);
    memcpy(data, a->bytes.data, a->bytes.len);
    memcpy(data + a->bytes.len, b->bytes.data, b->bytes.len);
    iris_value_t *r = iris_bytes(data, total);
    free(data);
    return r;
}

static iris_value_t *prim_bytes_len(iris_value_t *v) {
    if (v->type == IRIS_BYTES) return iris_int((int64_t)v->bytes.len);
    return iris_int(0);
}

/* -----------------------------------------------------------------------
 * Graph construction primitives
 * ----------------------------------------------------------------------- */

static iris_value_t *prim_graph_new_op(void) {
    iris_graph_t *g = iris_graph_new();
    iris_value_t *r = iris_program(g);
    iris_graph_release(g);
    return r;
}

static iris_value_t *prim_graph_add_node_rt(iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "graph_add_node_rt");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) return iris_unit();
    uint8_t kind = (uint8_t)iris_coerce_int(ARG(1));

    /* Generate a simple ID based on current count */
    uint64_t new_id = g->data->node_count + 1000000;
    iris_node_t node;
    memset(&node, 0, sizeof(node));
    node.id = new_id;
    node.kind = kind;
    iris_graph_add_node(g, node);

    /* Return (program, node_id) */
    iris_value_t *parts[2] = { ARG(0), iris_int((int64_t)new_id) };
    iris_value_t *r = iris_tuple(parts, 2);
    iris_release(parts[1]);
    return r;
}

static iris_value_t *prim_graph_connect(iris_value_t **args, size_t nargs) {
    NEED_ARGS(5, "graph_connect");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) { iris_retain(ARG(0)); return ARG(0); }
    iris_edge_t edge;
    edge.source = (uint64_t)iris_coerce_int(ARG(1));
    edge.target = (uint64_t)iris_coerce_int(ARG(2));
    edge.port = (uint8_t)iris_coerce_int(ARG(3));
    edge.label = (uint8_t)iris_coerce_int(ARG(4));
    iris_graph_add_edge(g, edge);
    iris_retain(ARG(0));
    return ARG(0);
}

static iris_value_t *prim_graph_set_prim_op(iris_value_t **args, size_t nargs) {
    NEED_ARGS(3, "graph_set_prim_op");
    iris_graph_t *g = extract_graph(ARG(0));
    if (!g) { iris_retain(ARG(0)); return ARG(0); }
    uint64_t nid = (uint64_t)iris_coerce_int(ARG(1));
    uint8_t opcode = (uint8_t)iris_coerce_int(ARG(2));
    iris_node_t *node = iris_graph_find_node(g, nid);
    if (node && node->kind == NK_PRIM) {
        node->payload.prim_opcode = opcode;
    }
    iris_retain(ARG(0));
    return ARG(0);
}

/* -----------------------------------------------------------------------
 * Map/State primitives
 * ----------------------------------------------------------------------- */

/* Maps are represented as tuples of (key, value) pairs */

static iris_value_t *prim_map_get(iris_value_t **args, size_t nargs) {
    NEED_ARGS(2, "map_get");
    /* Simple: unit for now */
    return iris_unit();
}

static iris_value_t *prim_map_insert(iris_value_t **args, size_t nargs) {
    NEED_ARGS(3, "map_insert");
    return iris_unit();
}

/* -----------------------------------------------------------------------
 * Main dispatch
 * ----------------------------------------------------------------------- */

iris_value_t *iris_dispatch_prim(iris_eval_ctx_t *ctx, uint8_t opcode,
                                 iris_value_t **args, size_t nargs) {
    switch (opcode) {
    /* Arithmetic */
    case 0x00: NEED_ARGS(2, "add"); return prim_arith_binop(ARG(0), ARG(1), op_add, fop_add);
    case 0x01: NEED_ARGS(2, "sub"); return prim_arith_binop(ARG(0), ARG(1), op_sub, fop_sub);
    case 0x02: NEED_ARGS(2, "mul"); return prim_arith_binop(ARG(0), ARG(1), op_mul, fop_mul);
    case 0x03: { /* div */
        NEED_ARGS(2, "div");
        if (iris_is_float_op2(ARG(0), ARG(1))) {
            double b = iris_coerce_f64(ARG(1));
            if (b == 0.0) return iris_int(0); /* division by zero -> 0 */
            return iris_f64(iris_coerce_f64(ARG(0)) / b);
        }
        int64_t b = iris_coerce_int(ARG(1));
        if (b == 0) return iris_int(0);
        return iris_int(iris_coerce_int(ARG(0)) / b);
    }
    case 0x04: { /* mod */
        NEED_ARGS(2, "mod");
        int64_t b = iris_coerce_int(ARG(1));
        if (b == 0) return iris_int(0);
        return iris_int(iris_coerce_int(ARG(0)) % b);
    }
    case 0x05: { /* neg */
        NEED_ARGS(1, "neg");
        if (ARG(0)->type == IRIS_F64) return iris_f64(-ARG(0)->f64);
        return iris_int(-iris_coerce_int(ARG(0)));
    }
    case 0x06: { /* abs */
        NEED_ARGS(1, "abs");
        if (ARG(0)->type == IRIS_F64) return iris_f64(fabs(ARG(0)->f64));
        int64_t n = iris_coerce_int(ARG(0));
        return iris_int(n < 0 ? -n : n);
    }
    case 0x07: NEED_ARGS(2, "min"); return prim_arith_binop(ARG(0), ARG(1), op_min, fop_min);
    case 0x08: NEED_ARGS(2, "max"); return prim_arith_binop(ARG(0), ARG(1), op_max, fop_max);
    case 0x09: { /* pow */
        NEED_ARGS(2, "pow");
        if (iris_is_float_op2(ARG(0), ARG(1))) {
            return iris_f64(pow(iris_coerce_f64(ARG(0)), iris_coerce_f64(ARG(1))));
        }
        int64_t base = iris_coerce_int(ARG(0));
        int64_t exp = iris_coerce_int(ARG(1));
        int64_t result = 1;
        for (int64_t i = 0; i < exp && i < 63; i++) result *= base;
        return iris_int(result);
    }

    /* Bitwise */
    case 0x10: NEED_ARGS(2, "and"); return iris_int(iris_coerce_int(ARG(0)) & iris_coerce_int(ARG(1)));
    case 0x11: NEED_ARGS(2, "or");  return iris_int(iris_coerce_int(ARG(0)) | iris_coerce_int(ARG(1)));
    case 0x12: NEED_ARGS(2, "xor"); return iris_int(iris_coerce_int(ARG(0)) ^ iris_coerce_int(ARG(1)));
    case 0x13: NEED_ARGS(1, "not"); return iris_int(~iris_coerce_int(ARG(0)));
    case 0x14: NEED_ARGS(2, "shl"); return iris_int(iris_coerce_int(ARG(0)) << (iris_coerce_int(ARG(1)) & 63));
    case 0x15: NEED_ARGS(2, "shr"); return iris_int((int64_t)((uint64_t)iris_coerce_int(ARG(0)) >> (iris_coerce_int(ARG(1)) & 63)));

    /* Comparison */
    case 0x20: NEED_ARGS(2, "eq"); return prim_cmp(ARG(0), ARG(1), cmp_eq);
    case 0x21: NEED_ARGS(2, "ne"); return prim_cmp(ARG(0), ARG(1), cmp_ne);
    case 0x22: NEED_ARGS(2, "lt"); return prim_cmp(ARG(0), ARG(1), cmp_lt);
    case 0x23: NEED_ARGS(2, "gt"); return prim_cmp(ARG(0), ARG(1), cmp_gt);
    case 0x24: NEED_ARGS(2, "le"); return prim_cmp(ARG(0), ARG(1), cmp_le);
    case 0x25: NEED_ARGS(2, "ge"); return prim_cmp(ARG(0), ARG(1), cmp_ge);

    /* zip */
    case 0x32: NEED_ARGS(2, "zip"); return prim_zip(ARG(0), ARG(1));

    /* List concat, reverse */
    case 0x35: NEED_ARGS(2, "list_concat"); return prim_list_concat(ARG(0), ARG(1));
    case 0x36: NEED_ARGS(1, "reverse"); return prim_list_reverse(ARG(0));

    /* Conversion */
    case 0x40: NEED_ARGS(1, "int_to_float"); return iris_f64((double)iris_coerce_int(ARG(0)));
    case 0x41: NEED_ARGS(1, "float_to_int"); return iris_int((int64_t)iris_coerce_f64(ARG(0)));
    case 0x42: { /* float_to_bits */
        NEED_ARGS(1, "float_to_bits");
        double f = iris_coerce_f64(ARG(0));
        uint64_t bits;
        memcpy(&bits, &f, 8);
        return iris_int((int64_t)bits);
    }
    case 0x43: { /* bits_to_float */
        NEED_ARGS(1, "bits_to_float");
        uint64_t bits = (uint64_t)iris_coerce_int(ARG(0));
        double f;
        memcpy(&f, &bits, 8);
        return iris_f64(f);
    }
    case 0x44: NEED_ARGS(1, "bool_to_int"); return iris_int(iris_coerce_int(ARG(0)));

    /* Map/State */
    case 0x50: return prim_map_get(args, nargs);
    case 0x51: return prim_map_insert(args, nargs);
    case 0x55: case 0x56: return iris_tuple_empty(); /* empty state */

    /* Graph introspection */
    case 0x80: /* graph_self */
        return iris_program(ctx->graph);
    case 0x81: NEED_ARGS(1, "graph_nodes"); return prim_graph_nodes(ARG(0));
    case 0x82: NEED_ARGS(2, "graph_get_kind"); return prim_graph_get_kind(ARG(0), ARG(1));
    case 0x83: NEED_ARGS(2, "graph_get_prim_op"); return prim_graph_get_prim_op(ARG(0), ARG(1));
    case 0x89: return prim_graph_eval(ctx, args, nargs);
    case 0x8A: NEED_ARGS(1, "graph_get_root"); return prim_graph_get_root(ARG(0));
    case 0x8F: NEED_ARGS(2, "graph_outgoing"); return prim_graph_outgoing(ARG(0), ARG(1));
    case 0x60: NEED_ARGS(2, "graph_get_node_cost"); return iris_int(0); /* stub */
    case 0x61: /* graph_set_node_type */ if (nargs > 0) { iris_retain(ARG(0)); return ARG(0); } return iris_unit();
    case 0x62: NEED_ARGS(2, "graph_get_node_type"); return iris_int(0); /* stub */
    case 0x63: { /* graph_edges */
        NEED_ARGS(1, "graph_edges");
        iris_graph_t *g = extract_graph(ARG(0));
        if (!g) return iris_tuple_empty();
        size_t n = g->data->edge_count;
        iris_value_t **elems = (iris_value_t **)malloc(sizeof(iris_value_t *) * n);
        for (size_t i = 0; i < n; i++) {
            iris_edge_t *e = &g->data->edges[i];
            iris_value_t *parts[4] = {
                iris_int((int64_t)e->source),
                iris_int((int64_t)e->target),
                iris_int((int64_t)e->port),
                iris_int((int64_t)e->label),
            };
            elems[i] = iris_tuple(parts, 4);
            for (int j = 0; j < 4; j++) iris_release(parts[j]);
        }
        iris_value_t *r = iris_tuple(elems, n);
        for (size_t i = 0; i < n; i++) iris_release(elems[i]);
        free(elems);
        return r;
    }
    case 0x64: NEED_ARGS(2, "graph_get_arity"); return prim_graph_get_arity(ARG(0), ARG(1));
    case 0x65: NEED_ARGS(2, "graph_get_depth"); return iris_int(0); /* stub */
    case 0x66: NEED_ARGS(2, "graph_get_lit_type_tag"); return prim_graph_get_lit_type_tag(ARG(0), ARG(1));
    case 0x96: NEED_ARGS(1, "graph_edge_count"); return prim_graph_edge_count_op(ARG(0));
    case 0x97: return prim_graph_edge_target_op(args, nargs);
    case 0x98: NEED_ARGS(2, "graph_get_binder"); return prim_graph_get_binder(ARG(0), ARG(1));
    case 0x99: return prim_graph_eval_env(ctx, args, nargs);
    case 0x9A: NEED_ARGS(2, "graph_get_tag"); return prim_graph_get_tag(ARG(0), ARG(1));
    case 0x9B: NEED_ARGS(2, "graph_get_field_index"); return prim_graph_get_field_index(ARG(0), ARG(1));
    case 0x9C: NEED_ARGS(1, "value_get_tag"); return prim_value_get_tag(ARG(0));
    case 0x9D: NEED_ARGS(1, "value_get_payload"); return prim_value_get_payload(ARG(0));
    case 0x9E: return prim_value_make_tagged(args, nargs);
    case 0x9F: NEED_ARGS(2, "graph_get_effect_tag"); return prim_graph_get_effect_tag(ARG(0), ARG(1));

    /* Graph mutation */
    case 0x84: return prim_graph_set_prim_op(args, nargs);
    case 0x85: return prim_graph_add_node_rt(args, nargs);
    case 0x86: return prim_graph_connect(args, nargs);
    case 0x8E: NEED_ARGS(2, "graph_get_lit_value"); return prim_graph_get_lit_value(ARG(0), ARG(1));
    case 0xED: return prim_graph_new_op();
    case 0xEE: NEED_ARGS(2, "graph_set_root"); return prim_graph_set_root_op(ARG(0), ARG(1));

    /* Par eval, spawn, identity */
    case 0x90: /* par_eval */ return prim_graph_eval(ctx, args, nargs);
    case 0x93: /* spawn */   return (nargs > 0) ? (iris_retain(ARG(0)), ARG(0)) : iris_unit();
    case 0x94: /* identity */ return (nargs > 0) ? (iris_retain(ARG(0)), ARG(0)) : iris_unit();

    /* String ops */
    case 0xB0: NEED_ARGS(1, "str_len"); return prim_str_len(ARG(0));
    case 0xB1: NEED_ARGS(2, "str_concat"); return prim_str_concat(ARG(0), ARG(1));
    case 0xB2: return prim_str_slice(args, nargs);
    case 0xB3: NEED_ARGS(2, "str_contains"); return prim_str_contains(ARG(0), ARG(1));
    case 0xB4: NEED_ARGS(2, "str_split"); return prim_str_split(ARG(0), ARG(1));
    case 0xB5: NEED_ARGS(2, "str_join"); return prim_str_join(ARG(0), ARG(1));
    case 0xB6: NEED_ARGS(1, "str_to_int"); return prim_str_to_int(ARG(0));
    case 0xB7: NEED_ARGS(1, "int_to_string"); return prim_int_to_string(ARG(0));
    case 0xB8: NEED_ARGS(2, "str_eq"); return prim_str_eq(ARG(0), ARG(1));
    case 0xB9: NEED_ARGS(2, "str_starts_with"); return prim_str_starts_with(ARG(0), ARG(1));
    case 0xBA: NEED_ARGS(2, "str_ends_with"); return prim_str_ends_with(ARG(0), ARG(1));
    case 0xBB: return prim_str_replace(args, nargs);
    case 0xBC: NEED_ARGS(1, "str_trim"); return prim_str_trim(ARG(0));
    case 0xBD: NEED_ARGS(1, "str_upper"); return prim_str_upper(ARG(0));
    case 0xBE: NEED_ARGS(1, "str_lower"); return prim_str_lower(ARG(0));
    case 0xBF: NEED_ARGS(1, "str_chars"); return prim_str_chars(ARG(0));
    case 0xC0: return prim_char_at(args, nargs);

    /* List ops */
    case 0xC1: NEED_ARGS(2, "list_append"); return prim_list_append(ARG(0), ARG(1));
    case 0xC2: NEED_ARGS(2, "list_nth"); return prim_list_nth(ARG(0), ARG(1));
    case 0xC3: NEED_ARGS(2, "list_take"); return prim_list_take(ARG(0), ARG(1));
    case 0xC4: NEED_ARGS(2, "list_drop"); return prim_list_drop(ARG(0), ARG(1));
    case 0xC5: NEED_ARGS(1, "list_sort"); return prim_list_sort(ARG(0));
    case 0xC6: NEED_ARGS(1, "list_dedup"); return prim_list_dedup(ARG(0));
    case 0xC7: return prim_list_range(args, nargs);
    case 0xCE: NEED_ARGS(2, "list_concat"); return prim_list_concat(ARG(0), ARG(1));
    case 0xF0: NEED_ARGS(1, "list_len"); return prim_list_len(ARG(0));

    /* Tuple ops */
    case 0xD2: NEED_ARGS(2, "tuple_get"); return prim_tuple_get(ARG(0), ARG(1));
    case 0xD6: NEED_ARGS(1, "tuple_len"); return prim_tuple_len(ARG(0));

    /* Bytes */
    case 0xE6: NEED_ARGS(1, "bytes_from_ints"); return prim_bytes_from_ints(ARG(0));
    case 0xE7: NEED_ARGS(2, "bytes_concat"); return prim_bytes_concat(ARG(0), ARG(1));
    case 0xE8: NEED_ARGS(1, "bytes_len"); return prim_bytes_len(ARG(0));

    /* Math */
    case 0xD8: NEED_ARGS(1, "sqrt"); return iris_f64(sqrt(iris_coerce_f64(ARG(0))));
    case 0xD9: NEED_ARGS(1, "log"); return iris_f64(log(iris_coerce_f64(ARG(0))));
    case 0xDA: NEED_ARGS(1, "exp"); return iris_f64(exp(iris_coerce_f64(ARG(0))));
    case 0xDB: NEED_ARGS(1, "sin"); return iris_f64(sin(iris_coerce_f64(ARG(0))));
    case 0xDC: NEED_ARGS(1, "cos"); return iris_f64(cos(iris_coerce_f64(ARG(0))));
    case 0xDD: NEED_ARGS(1, "floor"); return iris_f64(floor(iris_coerce_f64(ARG(0))));
    case 0xDE: NEED_ARGS(1, "ceil"); return iris_f64(ceil(iris_coerce_f64(ARG(0))));
    case 0xDF: NEED_ARGS(1, "round"); return iris_f64(round(iris_coerce_f64(ARG(0))));

    /* Constants */
    case 0xE0: return iris_f64(3.14159265358979323846);
    case 0xE1: return iris_f64(2.71828182845904523536);
    case 0xE2: { /* random_int */
        NEED_ARGS(1, "random_int");
        int64_t max = iris_coerce_int(ARG(0));
        if (max <= 0) return iris_int(0);
        return iris_int(rand() % max);
    }
    case 0xE3: return iris_f64((double)rand() / (double)RAND_MAX);

    /* Map ops */
    case 0xC8: return prim_map_insert(args, nargs);
    case 0xC9: return prim_map_get(args, nargs);
    case 0xCA: /* map_remove */ return iris_unit();
    case 0xCB: /* map_keys */ return iris_tuple_empty();
    case 0xCC: /* map_values */ return iris_tuple_empty();
    case 0xCD: /* map_size */ return iris_int(0);

    /* Effect */
    case 0xA1: { /* perform_effect */
        NEED_ARGS(2, "perform_effect");
        uint8_t etag = (uint8_t)iris_coerce_int(ARG(0));
        iris_value_t *eargs = ARG(1);
        if (eargs->type == IRIS_TUPLE) {
            return iris_dispatch_effect(etag, eargs->tuple.elems, eargs->tuple.len);
        }
        return iris_dispatch_effect(etag, &eargs, 1);
    }

    /* File read */
    case 0xF2: { /* file_read */
        NEED_ARGS(1, "file_read");
        if (ARG(0)->type != IRIS_STRING) return iris_string("");
        FILE *f = fopen(ARG(0)->str.data, "rb");
        if (!f) return iris_string("");
        fseek(f, 0, SEEK_END);
        long sz = ftell(f);
        fseek(f, 0, SEEK_SET);
        char *buf = (char *)malloc(sz + 1);
        size_t nread = fread(buf, 1, sz, f);
        buf[nread] = '\0';
        fclose(f);
        iris_value_t *r = iris_string_len(buf, nread);
        free(buf);
        return r;
    }

    /* Print */
    case 0xF4: {
        NEED_ARGS(1, "print");
        if (ARG(0)->type == IRIS_STRING) {
            fwrite(ARG(0)->str.data, 1, ARG(0)->str.len, stdout);
        } else {
            iris_print_value(ARG(0));
        }
        fflush(stdout);
        return iris_unit();
    }

    /* Buf ops (simple string builder) */
    case 0xD3: /* buf_new */ return iris_string("");
    case 0xD4: { /* buf_push */
        NEED_ARGS(2, "buf_push");
        return prim_str_concat(ARG(0), ARG(1));
    }
    case 0xD5: { /* buf_finish */
        NEED_ARGS(1, "buf_finish");
        iris_retain(ARG(0));
        return ARG(0);
    }

    /* Thunk force */
    case 0xEA: {
        NEED_ARGS(1, "thunk_force_eager");
        iris_retain(ARG(0));
        return ARG(0);
    }

    /* 0x34: list_nth (alias for tuple access) */
    case 0x34: NEED_ARGS(2, "list_nth"); return prim_list_nth(ARG(0), ARG(1));

    default:
        fprintf(stderr, "iris: unknown opcode 0x%02x\n", opcode);
        return iris_unit();
    }
}
