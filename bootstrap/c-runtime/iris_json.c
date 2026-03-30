/*
 * iris_json.c — Minimal recursive-descent JSON parser + graph loader
 *
 * Only handles the subset of JSON needed for interpreter.json:
 * - Objects, arrays, strings, integers, null, booleans
 * - Node IDs are uint64 (parsed from string keys and integer fields)
 * - Edge labels are strings ("Argument", "Scrutinee", etc.)
 *
 * No external dependencies.
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <ctype.h>
#include <errno.h>

#include "iris_json.h"

/* -----------------------------------------------------------------------
 * Mini JSON value representation (temporary, freed after loading)
 * ----------------------------------------------------------------------- */

typedef enum {
    JV_NULL, JV_BOOL, JV_INT, JV_FLOAT, JV_STRING, JV_ARRAY, JV_OBJECT
} jv_type_t;

typedef struct jv_pair jv_pair_t;
typedef struct jv jv_t;

struct jv {
    jv_type_t type;
    union {
        int          b;
        int64_t      i;
        double       f;
        struct { char *data; size_t len; } str;
        struct { jv_t **items; size_t len; size_t cap; } arr;
        struct { jv_pair_t *pairs; size_t len; size_t cap; } obj;
    };
};

struct jv_pair {
    char *key;
    jv_t *val;
};

/* -----------------------------------------------------------------------
 * JSON value lifecycle
 * ----------------------------------------------------------------------- */

static jv_t *jv_new(jv_type_t type) {
    jv_t *v = (jv_t *)calloc(1, sizeof(jv_t));
    v->type = type;
    return v;
}

static void jv_free(jv_t *v) {
    if (!v) return;
    switch (v->type) {
    case JV_STRING:
        free(v->str.data);
        break;
    case JV_ARRAY:
        for (size_t i = 0; i < v->arr.len; i++) jv_free(v->arr.items[i]);
        free(v->arr.items);
        break;
    case JV_OBJECT:
        for (size_t i = 0; i < v->obj.len; i++) {
            free(v->obj.pairs[i].key);
            jv_free(v->obj.pairs[i].val);
        }
        free(v->obj.pairs);
        break;
    default:
        break;
    }
    free(v);
}

static jv_t *jv_object_get(jv_t *obj, const char *key) {
    if (!obj || obj->type != JV_OBJECT) return NULL;
    for (size_t i = 0; i < obj->obj.len; i++) {
        if (strcmp(obj->obj.pairs[i].key, key) == 0)
            return obj->obj.pairs[i].val;
    }
    return NULL;
}

static int64_t jv_as_int(jv_t *v) {
    if (!v) return 0;
    if (v->type == JV_INT) return v->i;
    if (v->type == JV_FLOAT) return (int64_t)v->f;
    return 0;
}

static const char *jv_as_str(jv_t *v) {
    if (v && v->type == JV_STRING) return v->str.data;
    return "";
}

/* -----------------------------------------------------------------------
 * JSON parser (recursive descent)
 * ----------------------------------------------------------------------- */

typedef struct {
    const char *src;
    size_t pos;
    size_t len;
} parser_t;

static void skip_ws(parser_t *p) {
    while (p->pos < p->len && isspace((unsigned char)p->src[p->pos]))
        p->pos++;
}

static int peek(parser_t *p) {
    skip_ws(p);
    return p->pos < p->len ? p->src[p->pos] : -1;
}

static int next(parser_t *p) {
    skip_ws(p);
    return p->pos < p->len ? p->src[p->pos++] : -1;
}

static int expect(parser_t *p, char c) {
    if (next(p) != c) {
        fprintf(stderr, "json: expected '%c' at pos %zu\n", c, p->pos);
        return 0;
    }
    return 1;
}

static jv_t *parse_value(parser_t *p);

static jv_t *parse_string(parser_t *p) {
    if (!expect(p, '"')) return NULL;
    size_t start = p->pos;
    /* Simple: no escape handling beyond \\ and \" */
    while (p->pos < p->len && p->src[p->pos] != '"') {
        if (p->src[p->pos] == '\\') p->pos++; /* skip escaped char */
        p->pos++;
    }
    size_t slen = p->pos - start;
    jv_t *v = jv_new(JV_STRING);
    v->str.data = (char *)malloc(slen + 1);
    /* Copy with basic unescape */
    size_t j = 0;
    for (size_t i = start; i < start + slen; i++) {
        if (p->src[i] == '\\' && i + 1 < start + slen) {
            i++;
            switch (p->src[i]) {
            case '"':  v->str.data[j++] = '"'; break;
            case '\\': v->str.data[j++] = '\\'; break;
            case 'n':  v->str.data[j++] = '\n'; break;
            case 't':  v->str.data[j++] = '\t'; break;
            case 'r':  v->str.data[j++] = '\r'; break;
            case '/':  v->str.data[j++] = '/'; break;
            default:   v->str.data[j++] = p->src[i]; break;
            }
        } else {
            v->str.data[j++] = p->src[i];
        }
    }
    v->str.data[j] = '\0';
    v->str.len = j;
    p->pos++; /* skip closing " */
    return v;
}

static jv_t *parse_number(parser_t *p) {
    skip_ws(p);
    size_t start = p->pos;
    int is_float = 0;

    if (p->pos < p->len && p->src[p->pos] == '-') p->pos++;
    while (p->pos < p->len && isdigit((unsigned char)p->src[p->pos])) p->pos++;
    if (p->pos < p->len && p->src[p->pos] == '.') {
        is_float = 1;
        p->pos++;
        while (p->pos < p->len && isdigit((unsigned char)p->src[p->pos])) p->pos++;
    }
    if (p->pos < p->len && (p->src[p->pos] == 'e' || p->src[p->pos] == 'E')) {
        is_float = 1;
        p->pos++;
        if (p->pos < p->len && (p->src[p->pos] == '+' || p->src[p->pos] == '-'))
            p->pos++;
        while (p->pos < p->len && isdigit((unsigned char)p->src[p->pos])) p->pos++;
    }

    /* Parse the number text */
    char buf[64];
    size_t nlen = p->pos - start;
    if (nlen >= sizeof(buf)) nlen = sizeof(buf) - 1;
    memcpy(buf, &p->src[start], nlen);
    buf[nlen] = '\0';

    if (is_float) {
        jv_t *v = jv_new(JV_FLOAT);
        v->f = strtod(buf, NULL);
        return v;
    } else {
        jv_t *v = jv_new(JV_INT);
        /* Use strtoull for large uint64 values, then cast to int64 */
        errno = 0;
        if (buf[0] == '-') {
            v->i = (int64_t)strtoll(buf, NULL, 10);
        } else {
            v->i = (int64_t)strtoull(buf, NULL, 10);
        }
        return v;
    }
}

static jv_t *parse_array(parser_t *p) {
    if (!expect(p, '[')) return NULL;
    jv_t *v = jv_new(JV_ARRAY);
    v->arr.cap = 8;
    v->arr.items = (jv_t **)malloc(sizeof(jv_t *) * v->arr.cap);

    if (peek(p) == ']') { p->pos++; return v; }

    for (;;) {
        if (v->arr.len >= v->arr.cap) {
            v->arr.cap *= 2;
            v->arr.items = (jv_t **)realloc(v->arr.items,
                sizeof(jv_t *) * v->arr.cap);
        }
        v->arr.items[v->arr.len++] = parse_value(p);

        int c = peek(p);
        if (c == ',') { next(p); continue; }
        if (c == ']') { next(p); break; }
        fprintf(stderr, "json: unexpected char '%c' in array at %zu\n", c, p->pos);
        break;
    }
    return v;
}

static jv_t *parse_object(parser_t *p) {
    if (!expect(p, '{')) return NULL;
    jv_t *v = jv_new(JV_OBJECT);
    v->obj.cap = 8;
    v->obj.pairs = (jv_pair_t *)malloc(sizeof(jv_pair_t) * v->obj.cap);

    if (peek(p) == '}') { p->pos++; return v; }

    for (;;) {
        if (v->obj.len >= v->obj.cap) {
            v->obj.cap *= 2;
            v->obj.pairs = (jv_pair_t *)realloc(v->obj.pairs,
                sizeof(jv_pair_t) * v->obj.cap);
        }
        jv_t *key = parse_string(p);
        if (!key) break;
        if (!expect(p, ':')) { jv_free(key); break; }
        jv_t *val = parse_value(p);

        v->obj.pairs[v->obj.len].key = key->str.data;
        key->str.data = NULL; /* transfer ownership */
        jv_free(key);
        v->obj.pairs[v->obj.len].val = val;
        v->obj.len++;

        int c = peek(p);
        if (c == ',') { next(p); continue; }
        if (c == '}') { next(p); break; }
        fprintf(stderr, "json: unexpected char '%c' in object at %zu\n", c, p->pos);
        break;
    }
    return v;
}

static jv_t *parse_value(parser_t *p) {
    int c = peek(p);
    if (c == '"') return parse_string(p);
    if (c == '{') return parse_object(p);
    if (c == '[') return parse_array(p);
    if (c == '-' || isdigit(c)) return parse_number(p);
    if (c == 't') { p->pos += 4; jv_t *v = jv_new(JV_BOOL); v->b = 1; return v; }
    if (c == 'f') { p->pos += 5; jv_t *v = jv_new(JV_BOOL); v->b = 0; return v; }
    if (c == 'n') { p->pos += 4; return jv_new(JV_NULL); }
    fprintf(stderr, "json: unexpected char '%c' at pos %zu\n", c, p->pos);
    return jv_new(JV_NULL);
}

/* -----------------------------------------------------------------------
 * Graph loading from parsed JSON
 * ----------------------------------------------------------------------- */

static uint8_t parse_kind(const char *s) {
    if (strcmp(s, "Prim") == 0) return NK_PRIM;
    if (strcmp(s, "Apply") == 0) return NK_APPLY;
    if (strcmp(s, "Lambda") == 0) return NK_LAMBDA;
    if (strcmp(s, "Let") == 0) return NK_LET;
    if (strcmp(s, "Match") == 0) return NK_MATCH;
    if (strcmp(s, "Lit") == 0) return NK_LIT;
    if (strcmp(s, "Ref") == 0) return NK_REF;
    if (strcmp(s, "Neural") == 0) return NK_NEURAL;
    if (strcmp(s, "Fold") == 0) return NK_FOLD;
    if (strcmp(s, "Unfold") == 0) return NK_UNFOLD;
    if (strcmp(s, "Effect") == 0) return NK_EFFECT;
    if (strcmp(s, "Tuple") == 0) return NK_TUPLE;
    if (strcmp(s, "Inject") == 0) return NK_INJECT;
    if (strcmp(s, "Project") == 0) return NK_PROJECT;
    if (strcmp(s, "TypeAbst") == 0) return NK_TYPEABST;
    if (strcmp(s, "TypeApp") == 0) return NK_TYPEAPP;
    if (strcmp(s, "LetRec") == 0) return NK_LETREC;
    if (strcmp(s, "Guard") == 0) return NK_GUARD;
    if (strcmp(s, "Rewrite") == 0) return NK_REWRITE;
    if (strcmp(s, "Extern") == 0) return NK_EXTERN;
    fprintf(stderr, "json: unknown node kind '%s'\n", s);
    return 0xFF;
}

static uint8_t parse_edge_label(const char *s) {
    if (strcmp(s, "Argument") == 0) return EL_ARGUMENT;
    if (strcmp(s, "Scrutinee") == 0) return EL_SCRUTINEE;
    if (strcmp(s, "Binding") == 0) return EL_BINDING;
    if (strcmp(s, "Continuation") == 0) return EL_CONTINUATION;
    if (strcmp(s, "Decrease") == 0) return EL_DECREASE;
    fprintf(stderr, "json: unknown edge label '%s'\n", s);
    return 0;
}

static void load_node(iris_graph_t *g, jv_t *node_json) {
    iris_node_t node;
    memset(&node, 0, sizeof(node));

    node.id = (uint64_t)jv_as_int(jv_object_get(node_json, "id"));
    node.kind = parse_kind(jv_as_str(jv_object_get(node_json, "kind")));
    jv_t *arity_jv = jv_object_get(node_json, "arity");
    node.arity = arity_jv ? (uint8_t)jv_as_int(arity_jv) : 0;

    /* Parse payload based on kind */
    jv_t *payload = jv_object_get(node_json, "payload");
    if (!payload || payload->type != JV_OBJECT) goto done;

    switch (node.kind) {
    case NK_PRIM: {
        jv_t *prim = jv_object_get(payload, "Prim");
        if (prim) {
            node.payload.prim_opcode = (uint8_t)jv_as_int(jv_object_get(prim, "opcode"));
        }
        break;
    }
    case NK_LIT: {
        jv_t *lit = jv_object_get(payload, "Lit");
        if (lit) {
            node.payload.lit.type_tag = (uint8_t)jv_as_int(jv_object_get(lit, "type_tag"));
            jv_t *val_arr = jv_object_get(lit, "value");
            if (val_arr && val_arr->type == JV_ARRAY) {
                size_t n = val_arr->arr.len;
                node.payload.lit.value_len = n;
                if (n > 0) {
                    node.payload.lit.value = (uint8_t *)malloc(n);
                    for (size_t i = 0; i < n; i++) {
                        node.payload.lit.value[i] = (uint8_t)jv_as_int(val_arr->arr.items[i]);
                    }
                }
            }
        }
        break;
    }
    case NK_GUARD: {
        jv_t *guard = jv_object_get(payload, "Guard");
        if (guard) {
            node.payload.guard.pred = (uint64_t)jv_as_int(jv_object_get(guard, "predicate_node"));
            node.payload.guard.body = (uint64_t)jv_as_int(jv_object_get(guard, "body_node"));
            node.payload.guard.fallback = (uint64_t)jv_as_int(jv_object_get(guard, "fallback_node"));
        }
        break;
    }
    case NK_LAMBDA: {
        jv_t *lam = jv_object_get(payload, "Lambda");
        if (lam) {
            node.payload.lambda.binder_id = (uint32_t)jv_as_int(jv_object_get(lam, "binder"));
            jv_t *cc = jv_object_get(lam, "captured_count");
            node.payload.lambda.captured_count = cc ? (uint32_t)jv_as_int(cc) : 0;
        }
        break;
    }
    case NK_INJECT: {
        jv_t *inj = jv_object_get(payload, "Inject");
        if (inj) {
            node.payload.inject.tag_index = (uint16_t)jv_as_int(jv_object_get(inj, "tag_index"));
        }
        break;
    }
    case NK_PROJECT: {
        jv_t *proj = jv_object_get(payload, "Project");
        if (proj) {
            node.payload.project.field_index = (uint16_t)jv_as_int(jv_object_get(proj, "field_index"));
        }
        break;
    }
    case NK_EFFECT: {
        jv_t *eff = jv_object_get(payload, "Effect");
        if (eff) {
            node.payload.effect_tag = (uint8_t)jv_as_int(jv_object_get(eff, "effect_tag"));
        }
        break;
    }
    case NK_LETREC: {
        jv_t *lr = jv_object_get(payload, "LetRec");
        if (lr) {
            node.payload.letrec.binder_id = (uint32_t)jv_as_int(jv_object_get(lr, "binder"));
        }
        break;
    }
    case NK_MATCH: {
        jv_t *m = jv_object_get(payload, "Match");
        if (m) {
            node.payload.match_info.arm_count = (uint16_t)jv_as_int(jv_object_get(m, "arm_count"));
        }
        break;
    }
    default:
        /* Apply, Let, Tuple, Fold, Unfold, Ref, etc. — no payload fields needed */
        break;
    }

done:
    iris_graph_add_node(g, node);
}

static void load_edge(iris_graph_t *g, jv_t *edge_json) {
    iris_edge_t edge;
    memset(&edge, 0, sizeof(edge));
    edge.source = (uint64_t)jv_as_int(jv_object_get(edge_json, "source"));
    edge.target = (uint64_t)jv_as_int(jv_object_get(edge_json, "target"));
    edge.port = (uint8_t)jv_as_int(jv_object_get(edge_json, "port"));

    jv_t *label = jv_object_get(edge_json, "label");
    if (label && label->type == JV_STRING) {
        edge.label = parse_edge_label(label->str.data);
    }

    iris_graph_add_edge(g, edge);
}

/* -----------------------------------------------------------------------
 * Public API
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_load_json(const char *path) {
    FILE *f = fopen(path, "rb");
    if (!f) {
        fprintf(stderr, "iris_load_json: cannot open '%s'\n", path);
        return NULL;
    }
    fseek(f, 0, SEEK_END);
    long fsize = ftell(f);
    fseek(f, 0, SEEK_SET);

    char *buf = (char *)malloc(fsize + 1);
    if (!buf) { fclose(f); return NULL; }
    size_t nread = fread(buf, 1, fsize, f);
    buf[nread] = '\0';
    fclose(f);

    parser_t p = { .src = buf, .pos = 0, .len = (size_t)fsize };
    jv_t *root = parse_value(&p);
    free(buf);

    if (!root || root->type != JV_OBJECT) {
        fprintf(stderr, "iris_load_json: invalid JSON\n");
        jv_free(root);
        return NULL;
    }

    iris_graph_t *g = iris_graph_new();

    /* Root node ID */
    jv_t *root_id = jv_object_get(root, "root");
    if (root_id) g->root = (uint64_t)jv_as_int(root_id);

    /* Nodes: object mapping string IDs -> node objects */
    jv_t *nodes_obj = jv_object_get(root, "nodes");
    if (nodes_obj && nodes_obj->type == JV_OBJECT) {
        for (size_t i = 0; i < nodes_obj->obj.len; i++) {
            load_node(g, nodes_obj->obj.pairs[i].val);
        }
    }

    /* Edges: array of edge objects */
    jv_t *edges_arr = jv_object_get(root, "edges");
    if (edges_arr && edges_arr->type == JV_ARRAY) {
        for (size_t i = 0; i < edges_arr->arr.len; i++) {
            load_edge(g, edges_arr->arr.items[i]);
        }
    }

    jv_free(root);
    return g;
}
