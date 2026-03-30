/*
<<<<<<< HEAD
 * iris_graph.c — SemanticGraph data structures
 *
 * Implements COW (copy-on-write) graph wrappers: multiple iris_graph_t
 * can share the same underlying iris_graph_data_t. Node/edge lookup,
 * argument collection, and construction APIs.
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>

#include "iris_graph.h"

#define INITIAL_NODE_CAP 64
#define INITIAL_EDGE_CAP 64

/* -----------------------------------------------------------------------
 * Graph data lifecycle
 * ----------------------------------------------------------------------- */

static iris_graph_data_t *graph_data_new(void) {
    iris_graph_data_t *d = (iris_graph_data_t *)calloc(1, sizeof(iris_graph_data_t));
    if (!d) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    d->refcount = 1;
    d->node_capacity = INITIAL_NODE_CAP;
    d->nodes = (iris_node_t *)calloc(d->node_capacity, sizeof(iris_node_t));
    d->edge_capacity = INITIAL_EDGE_CAP;
    d->edges = (iris_edge_t *)calloc(d->edge_capacity, sizeof(iris_edge_t));
    if (!d->nodes || !d->edges) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    return d;
}

static void graph_data_retain(iris_graph_data_t *d) {
    if (d) d->refcount++;
}

static void graph_data_release(iris_graph_data_t *d) {
    if (!d) return;
    if (d->refcount > 1) { d->refcount--; return; }
    /* Free lit value buffers in nodes */
    for (size_t i = 0; i < d->node_count; i++) {
        if (d->nodes[i].kind == NK_LIT && d->nodes[i].payload.lit.value) {
            free(d->nodes[i].payload.lit.value);
        }
    }
    free(d->nodes);
    free(d->edges);
    free(d);
}

/* -----------------------------------------------------------------------
 * Graph wrapper lifecycle
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_new(void) {
    iris_graph_t *g = (iris_graph_t *)calloc(1, sizeof(iris_graph_t));
    if (!g) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    g->refcount = 1;
    g->root = 0;
    g->data = graph_data_new();
    return g;
}

void iris_graph_retain(iris_graph_t *g) {
    if (g) g->refcount++;
}

void iris_graph_release(iris_graph_t *g) {
    if (!g) return;
    if (g->refcount > 1) { g->refcount--; return; }
    graph_data_release(g->data);
    free(g);
}

/* -----------------------------------------------------------------------
 * Node/edge construction
 * ----------------------------------------------------------------------- */

void iris_graph_add_node(iris_graph_t *g, iris_node_t node) {
    iris_graph_data_t *d = g->data;
    if (d->node_count >= d->node_capacity) {
        d->node_capacity *= 2;
        d->nodes = (iris_node_t *)realloc(d->nodes,
            d->node_capacity * sizeof(iris_node_t));
        if (!d->nodes) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    }
    d->nodes[d->node_count++] = node;
}

void iris_graph_add_edge(iris_graph_t *g, iris_edge_t edge) {
    iris_graph_data_t *d = g->data;
    if (d->edge_count >= d->edge_capacity) {
        d->edge_capacity *= 2;
        d->edges = (iris_edge_t *)realloc(d->edges,
            d->edge_capacity * sizeof(iris_edge_t));
        if (!d->edges) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    }
    d->edges[d->edge_count++] = edge;
}

/* -----------------------------------------------------------------------
 * Node lookup (linear scan — fine for <1000 nodes)
 * ----------------------------------------------------------------------- */

iris_node_t *iris_graph_find_node(iris_graph_t *g, uint64_t id) {
    iris_graph_data_t *d = g->data;
    for (size_t i = 0; i < d->node_count; i++) {
        if (d->nodes[i].id == id) return &d->nodes[i];
=======
 * iris_graph.c -- Low-level graph operations and JSON loader.
 */

#include "iris_graph.h"
#include "iris_rt.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* -----------------------------------------------------------------------
 * Graph allocation
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_raw_alloc(void) {
    iris_graph_t *g = calloc(1, sizeof(iris_graph_t));
    if (!g) { fprintf(stderr, "out of memory\n"); abort(); }
    g->node_cap  = 512;
    g->nodes     = calloc(g->node_cap, sizeof(iris_node_t));
    g->edge_cap  = 512;
    g->edges     = calloc(g->edge_cap, sizeof(iris_edge_t));
    return g;
}

void iris_graph_raw_free(iris_graph_t *g) {
    if (!g) return;
    for (uint32_t i = 0; i < g->node_count; i++) {
        iris_node_t *n = &g->nodes[i];
        if (n->kind == NK_LIT && n->payload.lit.value) {
            free(n->payload.lit.value);
        }
        if (n->kind == NK_FOLD && n->payload.fold.descriptor) {
            free(n->payload.fold.descriptor);
        }
    }
    free(g->nodes);
    free(g->edges);
    free(g);
}

iris_graph_t *iris_graph_raw_clone(const iris_graph_t *g) {
    if (!g) return NULL;
    iris_graph_t *c = calloc(1, sizeof(iris_graph_t));
    if (!c) { fprintf(stderr, "out of memory\n"); abort(); }
    c->root       = g->root;
    c->node_count = g->node_count;
    c->node_cap   = g->node_count > 0 ? g->node_count : 1;
    c->nodes      = calloc(c->node_cap, sizeof(iris_node_t));
    memcpy(c->nodes, g->nodes, g->node_count * sizeof(iris_node_t));
    /* Deep-copy heap payloads */
    for (uint32_t i = 0; i < c->node_count; i++) {
        iris_node_t *n = &c->nodes[i];
        if (n->kind == NK_LIT && n->payload.lit.value && n->payload.lit.value_len > 0) {
            uint8_t *copy = malloc(n->payload.lit.value_len);
            memcpy(copy, g->nodes[i].payload.lit.value, n->payload.lit.value_len);
            n->payload.lit.value = copy;
        }
        if (n->kind == NK_FOLD && n->payload.fold.descriptor && n->payload.fold.descriptor_len > 0) {
            uint8_t *copy = malloc(n->payload.fold.descriptor_len);
            memcpy(copy, g->nodes[i].payload.fold.descriptor, n->payload.fold.descriptor_len);
            n->payload.fold.descriptor = copy;
        }
    }
    c->edge_count = g->edge_count;
    c->edge_cap   = g->edge_count > 0 ? g->edge_count : 1;
    c->edges      = calloc(c->edge_cap, sizeof(iris_edge_t));
    memcpy(c->edges, g->edges, g->edge_count * sizeof(iris_edge_t));
    return c;
}

/* -----------------------------------------------------------------------
 * Node / edge insertion
 * ----------------------------------------------------------------------- */

void iris_graph_raw_add_node(iris_graph_t *g, iris_node_t node) {
    if (g->node_count >= g->node_cap) {
        g->node_cap *= 2;
        g->nodes = realloc(g->nodes, g->node_cap * sizeof(iris_node_t));
        if (!g->nodes) { fprintf(stderr, "out of memory\n"); abort(); }
    }
    g->nodes[g->node_count++] = node;
}

void iris_graph_raw_add_edge(iris_graph_t *g, iris_edge_t edge) {
    if (g->edge_count >= g->edge_cap) {
        g->edge_cap *= 2;
        g->edges = realloc(g->edges, g->edge_cap * sizeof(iris_edge_t));
        if (!g->edges) { fprintf(stderr, "out of memory\n"); abort(); }
    }
    g->edges[g->edge_count++] = edge;
}

/* -----------------------------------------------------------------------
 * Node lookup (linear scan; fine for ~400 nodes)
 * ----------------------------------------------------------------------- */

iris_node_t *iris_graph_raw_find_node(const iris_graph_t *g, uint64_t id) {
    for (uint32_t i = 0; i < g->node_count; i++) {
        if (g->nodes[i].id == id) return &g->nodes[i];
>>>>>>> worktree-agent-a01b95aa
    }
    return NULL;
}

/* -----------------------------------------------------------------------
<<<<<<< HEAD
 * Edge queries
 * ----------------------------------------------------------------------- */

size_t iris_graph_find_edges(iris_graph_t *g, uint64_t source,
                             iris_edge_t **out) {
    iris_graph_data_t *d = g->data;
    /* Count first */
    size_t count = 0;
    for (size_t i = 0; i < d->edge_count; i++) {
        if (d->edges[i].source == source) count++;
    }
    if (count == 0) { *out = NULL; return 0; }

    *out = (iris_edge_t *)malloc(count * sizeof(iris_edge_t));
    if (!*out) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    size_t j = 0;
    for (size_t i = 0; i < d->edge_count; i++) {
        if (d->edges[i].source == source) {
            (*out)[j++] = d->edges[i];
        }
    }
    return count;
}

uint64_t iris_graph_edge_target(iris_graph_t *g, uint64_t source,
                                uint8_t port, uint8_t label) {
    iris_graph_data_t *d = g->data;
    for (size_t i = 0; i < d->edge_count; i++) {
        if (d->edges[i].source == source &&
            d->edges[i].port == port &&
            d->edges[i].label == label) {
            return d->edges[i].target;
        }
    }
    return 0; /* sentinel: not found */
}

size_t iris_graph_node_count(iris_graph_t *g) {
    return g->data->node_count;
}

size_t iris_graph_edge_count(iris_graph_t *g) {
    return g->data->edge_count;
}

/* -----------------------------------------------------------------------
 * Argument collection: find Argument edges from source, sorted by port
 * ----------------------------------------------------------------------- */

size_t iris_graph_argument_targets(iris_graph_t *g, uint64_t source,
                                   uint64_t *out, size_t max) {
    iris_graph_data_t *d = g->data;
    /* Collect (port, target) pairs for Argument edges */
    size_t count = 0;
    uint8_t ports[32];
    uint64_t targets[32];

    for (size_t i = 0; i < d->edge_count && count < 32; i++) {
        if (d->edges[i].source == source &&
            d->edges[i].label == EL_ARGUMENT) {
            ports[count] = d->edges[i].port;
            targets[count] = d->edges[i].target;
            count++;
        }
    }

    /* Sort by port (insertion sort — count is small) */
    for (size_t i = 1; i < count; i++) {
        uint8_t pk = ports[i];
        uint64_t tk = targets[i];
        size_t j = i;
        while (j > 0 && ports[j - 1] > pk) {
            ports[j] = ports[j - 1];
            targets[j] = targets[j - 1];
            j--;
        }
        ports[j] = pk;
        targets[j] = tk;
    }

    size_t n = count < max ? count : max;
    for (size_t i = 0; i < n; i++) {
        out[i] = targets[i];
    }
    return n;
}

/* -----------------------------------------------------------------------
 * COW: new wrapper with different root, sharing data
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_set_root(iris_graph_t *g, uint64_t new_root) {
    iris_graph_t *w = (iris_graph_t *)calloc(1, sizeof(iris_graph_t));
    if (!w) { fprintf(stderr, "iris: out of memory\n"); abort(); }
    w->refcount = 1;
    w->root = new_root;
    w->data = g->data;
    graph_data_retain(g->data);
    return w;
=======
 * Outgoing edges: collect targets with the given label, sorted by port.
 * Returns count written to `out`.
 * ----------------------------------------------------------------------- */

uint32_t iris_graph_raw_outgoing(const iris_graph_t *g, uint64_t source,
                                 uint8_t label, uint64_t *out, uint32_t out_cap) {
    /* Collect matching edges into a temp buffer, then sort by port */
    typedef struct { uint8_t port; uint64_t target; } pe_t;
    pe_t buf[256];
    uint32_t count = 0;

    for (uint32_t i = 0; i < g->edge_count; i++) {
        const iris_edge_t *e = &g->edges[i];
        if (e->source == source && e->label == label && count < 256) {
            buf[count].port   = e->port;
            buf[count].target = e->target;
            count++;
        }
    }
    /* Insertion sort by port (count is small) */
    for (uint32_t i = 1; i < count; i++) {
        pe_t key = buf[i];
        int j = (int)i - 1;
        while (j >= 0 && buf[j].port > key.port) {
            buf[j + 1] = buf[j];
            j--;
        }
        buf[j + 1] = key;
    }
    uint32_t written = count < out_cap ? count : out_cap;
    for (uint32_t i = 0; i < written; i++) {
        out[i] = buf[i].target;
    }
    return written;
}

/* -----------------------------------------------------------------------
 * Single edge target lookup
 * ----------------------------------------------------------------------- */

int iris_graph_raw_edge_target(const iris_graph_t *g, uint64_t source,
                               uint8_t port, uint8_t label,
                               uint64_t *out_target) {
    for (uint32_t i = 0; i < g->edge_count; i++) {
        const iris_edge_t *e = &g->edges[i];
        if (e->source == source && e->port == port && e->label == label) {
            *out_target = e->target;
            return 1;
        }
    }
    return 0;
}

/* -----------------------------------------------------------------------
 * Minimal JSON parser — just enough to load SemanticGraph JSON.
 *
 * We parse the subset produced by serde_json for the Rust SemanticGraph.
 * This is NOT a general-purpose JSON parser.
 * ----------------------------------------------------------------------- */

typedef struct {
    const char *data;
    size_t      pos;
    size_t      len;
} json_ctx_t;

static void json_skip_ws(json_ctx_t *ctx) {
    while (ctx->pos < ctx->len) {
        char c = ctx->data[ctx->pos];
        if (c == ' ' || c == '\t' || c == '\n' || c == '\r')
            ctx->pos++;
        else
            break;
    }
}

static int json_peek(json_ctx_t *ctx) {
    json_skip_ws(ctx);
    if (ctx->pos >= ctx->len) return -1;
    return (unsigned char)ctx->data[ctx->pos];
}

static int json_expect(json_ctx_t *ctx, char c) {
    json_skip_ws(ctx);
    if (ctx->pos < ctx->len && ctx->data[ctx->pos] == c) {
        ctx->pos++;
        return 1;
    }
    return 0;
}

/* Parse a JSON string, writing into buf (null-terminated).
 * Returns length of string (excluding null terminator), or -1 on error. */
static int json_parse_string(json_ctx_t *ctx, char *buf, size_t buf_cap) {
    json_skip_ws(ctx);
    if (ctx->pos >= ctx->len || ctx->data[ctx->pos] != '"') return -1;
    ctx->pos++; /* skip opening quote */
    size_t out = 0;
    while (ctx->pos < ctx->len) {
        char c = ctx->data[ctx->pos++];
        if (c == '"') {
            if (buf && out < buf_cap) buf[out] = '\0';
            return (int)out;
        }
        if (c == '\\' && ctx->pos < ctx->len) {
            char esc = ctx->data[ctx->pos++];
            switch (esc) {
                case '"': c = '"'; break;
                case '\\': c = '\\'; break;
                case '/': c = '/'; break;
                case 'n': c = '\n'; break;
                case 't': c = '\t'; break;
                case 'r': c = '\r'; break;
                case 'b': c = '\b'; break;
                case 'f': c = '\f'; break;
                case 'u':
                    /* Skip 4 hex digits (we don't need unicode for graph JSON) */
                    ctx->pos += 4;
                    c = '?';
                    break;
                default: c = esc; break;
            }
        }
        if (buf && out < buf_cap - 1) buf[out] = c;
        out++;
    }
    return -1; /* unterminated string */
}

/* Skip a JSON string without storing it */
static int json_skip_string(json_ctx_t *ctx) {
    return json_parse_string(ctx, NULL, 0);
}

/* Parse a JSON number as uint64_t */
static uint64_t json_parse_uint64(json_ctx_t *ctx) {
    json_skip_ws(ctx);
    uint64_t val = 0;
    int neg = 0;
    if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '-') {
        neg = 1;
        ctx->pos++;
    }
    while (ctx->pos < ctx->len) {
        char c = ctx->data[ctx->pos];
        if (c >= '0' && c <= '9') {
            val = val * 10 + (uint64_t)(c - '0');
            ctx->pos++;
        } else {
            break;
        }
    }
    /* Handle fractional part (skip it) */
    if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '.') {
        ctx->pos++;
        while (ctx->pos < ctx->len && ctx->data[ctx->pos] >= '0' && ctx->data[ctx->pos] <= '9')
            ctx->pos++;
    }
    /* Handle exponent (skip it) */
    if (ctx->pos < ctx->len && (ctx->data[ctx->pos] == 'e' || ctx->data[ctx->pos] == 'E')) {
        ctx->pos++;
        if (ctx->pos < ctx->len && (ctx->data[ctx->pos] == '+' || ctx->data[ctx->pos] == '-'))
            ctx->pos++;
        while (ctx->pos < ctx->len && ctx->data[ctx->pos] >= '0' && ctx->data[ctx->pos] <= '9')
            ctx->pos++;
    }
    return neg ? (uint64_t)(-(int64_t)val) : val;
}

/* Parse a JSON integer (signed) — currently unused but kept for future use */
#if 0
static int64_t json_parse_int64(json_ctx_t *ctx) {
    json_skip_ws(ctx);
    int64_t sign = 1;
    if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '-') {
        sign = -1;
        ctx->pos++;
    }
    int64_t val = 0;
    while (ctx->pos < ctx->len) {
        char c = ctx->data[ctx->pos];
        if (c >= '0' && c <= '9') {
            val = val * 10 + (int64_t)(c - '0');
            ctx->pos++;
        } else {
            break;
        }
    }
    return sign * val;
}
#endif

/* Skip an arbitrary JSON value */
static void json_skip_value(json_ctx_t *ctx) {
    json_skip_ws(ctx);
    if (ctx->pos >= ctx->len) return;
    char c = ctx->data[ctx->pos];
    if (c == '"') {
        json_skip_string(ctx);
    } else if (c == '{') {
        ctx->pos++;
        json_skip_ws(ctx);
        if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '}') {
            ctx->pos++;
            return;
        }
        while (1) {
            json_skip_string(ctx); /* key */
            json_expect(ctx, ':');
            json_skip_value(ctx);  /* value */
            json_skip_ws(ctx);
            if (ctx->pos < ctx->len && ctx->data[ctx->pos] == ',') {
                ctx->pos++;
            } else {
                break;
            }
        }
        json_expect(ctx, '}');
    } else if (c == '[') {
        ctx->pos++;
        json_skip_ws(ctx);
        if (ctx->pos < ctx->len && ctx->data[ctx->pos] == ']') {
            ctx->pos++;
            return;
        }
        while (1) {
            json_skip_value(ctx);
            json_skip_ws(ctx);
            if (ctx->pos < ctx->len && ctx->data[ctx->pos] == ',') {
                ctx->pos++;
            } else {
                break;
            }
        }
        json_expect(ctx, ']');
    } else if (c == 't') { /* true */
        ctx->pos += 4;
    } else if (c == 'f') { /* false */
        ctx->pos += 5;
    } else if (c == 'n') { /* null */
        ctx->pos += 4;
    } else {
        /* number */
        if (c == '-') ctx->pos++;
        while (ctx->pos < ctx->len && ctx->data[ctx->pos] >= '0' && ctx->data[ctx->pos] <= '9')
            ctx->pos++;
        if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '.') {
            ctx->pos++;
            while (ctx->pos < ctx->len && ctx->data[ctx->pos] >= '0' && ctx->data[ctx->pos] <= '9')
                ctx->pos++;
        }
        if (ctx->pos < ctx->len && (ctx->data[ctx->pos] == 'e' || ctx->data[ctx->pos] == 'E')) {
            ctx->pos++;
            if (ctx->pos < ctx->len && (ctx->data[ctx->pos] == '+' || ctx->data[ctx->pos] == '-'))
                ctx->pos++;
            while (ctx->pos < ctx->len && ctx->data[ctx->pos] >= '0' && ctx->data[ctx->pos] <= '9')
                ctx->pos++;
        }
    }
}

/* -----------------------------------------------------------------------
 * Parse NodeKind from string
 * ----------------------------------------------------------------------- */

static uint8_t parse_node_kind(const char *s) {
    if (strcmp(s, "Prim")     == 0) return NK_PRIM;
    if (strcmp(s, "Apply")    == 0) return NK_APPLY;
    if (strcmp(s, "Lambda")   == 0) return NK_LAMBDA;
    if (strcmp(s, "Let")      == 0) return NK_LET;
    if (strcmp(s, "Match")    == 0) return NK_MATCH;
    if (strcmp(s, "Lit")      == 0) return NK_LIT;
    if (strcmp(s, "Ref")      == 0) return NK_REF;
    if (strcmp(s, "Neural")   == 0) return NK_NEURAL;
    if (strcmp(s, "Fold")     == 0) return NK_FOLD;
    if (strcmp(s, "Unfold")   == 0) return NK_UNFOLD;
    if (strcmp(s, "Effect")   == 0) return NK_EFFECT;
    if (strcmp(s, "Tuple")    == 0) return NK_TUPLE;
    if (strcmp(s, "Inject")   == 0) return NK_INJECT;
    if (strcmp(s, "Project")  == 0) return NK_PROJECT;
    if (strcmp(s, "TypeAbst") == 0) return NK_TYPE_ABST;
    if (strcmp(s, "TypeApp")  == 0) return NK_TYPE_APP;
    if (strcmp(s, "LetRec")   == 0) return NK_LET_REC;
    if (strcmp(s, "Guard")    == 0) return NK_GUARD;
    if (strcmp(s, "Rewrite")  == 0) return NK_REWRITE;
    if (strcmp(s, "Extern")   == 0) return NK_EXTERN;
    fprintf(stderr, "warning: unknown node kind '%s'\n", s);
    return 0xFF;
}

/* -----------------------------------------------------------------------
 * Parse EdgeLabel from string
 * ----------------------------------------------------------------------- */

static uint8_t parse_edge_label(const char *s) {
    if (strcmp(s, "Argument")     == 0) return EL_ARGUMENT;
    if (strcmp(s, "Scrutinee")    == 0) return EL_SCRUTINEE;
    if (strcmp(s, "Binding")      == 0) return EL_BINDING;
    if (strcmp(s, "Continuation") == 0) return EL_CONTINUATION;
    if (strcmp(s, "Decrease")     == 0) return EL_DECREASE;
    fprintf(stderr, "warning: unknown edge label '%s'\n", s);
    return 0;
}

/* -----------------------------------------------------------------------
 * Parse a single node from JSON.
 * ctx should be positioned just after the opening '{' of the node object.
 * ----------------------------------------------------------------------- */

static iris_node_t parse_node(json_ctx_t *ctx) {
    iris_node_t node;
    memset(&node, 0, sizeof(node));
    char key[64];

    while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
        json_parse_string(ctx, key, sizeof(key));
        json_expect(ctx, ':');

        if (strcmp(key, "id") == 0) {
            node.id = json_parse_uint64(ctx);
        } else if (strcmp(key, "kind") == 0) {
            char val[32];
            json_parse_string(ctx, val, sizeof(val));
            node.kind = parse_node_kind(val);
        } else if (strcmp(key, "arity") == 0) {
            node.arity = (uint8_t)json_parse_uint64(ctx);
        } else if (strcmp(key, "type_sig") == 0) {
            node.type_sig = json_parse_uint64(ctx);
        } else if (strcmp(key, "salt") == 0) {
            node.salt = json_parse_uint64(ctx);
        } else if (strcmp(key, "payload") == 0) {
            /* payload is {"Kind": { ... }} OR "Kind" (string for unit variants) */
            json_skip_ws(ctx);
            if (ctx->pos < ctx->len && ctx->data[ctx->pos] == '"') {
                /* Unit variant like "Tuple", "Apply", "Let" */
                char ptype[32];
                json_parse_string(ctx, ptype, sizeof(ptype));
                /* No further payload to parse — node.kind was already set */
                if (json_peek(ctx) == ',') json_expect(ctx, ',');
                continue;
            }
            json_expect(ctx, '{');
            char ptype[32];
            json_parse_string(ctx, ptype, sizeof(ptype));
            json_expect(ctx, ':');

            if (strcmp(ptype, "Prim") == 0) {
                /* {"opcode": N} */
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "opcode") == 0) {
                        node.payload.prim_opcode = (uint8_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Lit") == 0) {
                /* {"type_tag": N, "value": [bytes...]} */
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "type_tag") == 0) {
                        node.payload.lit.type_tag = (uint8_t)json_parse_uint64(ctx);
                    } else if (strcmp(pk, "value") == 0) {
                        /* Array of byte values */
                        json_expect(ctx, '[');
                        uint8_t tmp[256];
                        uint32_t cnt = 0;
                        while (json_peek(ctx) != ']' && json_peek(ctx) != -1) {
                            if (cnt < sizeof(tmp))
                                tmp[cnt] = (uint8_t)json_parse_uint64(ctx);
                            cnt++;
                            if (json_peek(ctx) == ',') json_expect(ctx, ',');
                        }
                        json_expect(ctx, ']');
                        if (cnt > 0) {
                            node.payload.lit.value = malloc(cnt);
                            memcpy(node.payload.lit.value, tmp, cnt < sizeof(tmp) ? cnt : sizeof(tmp));
                            node.payload.lit.value_len = cnt;
                        }
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Guard") == 0) {
                /* {"predicate_node": N, "body_node": N, "fallback_node": N} */
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[64];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "predicate_node") == 0) {
                        node.payload.guard.predicate_node = json_parse_uint64(ctx);
                    } else if (strcmp(pk, "body_node") == 0) {
                        node.payload.guard.body_node = json_parse_uint64(ctx);
                    } else if (strcmp(pk, "fallback_node") == 0) {
                        node.payload.guard.fallback_node = json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Lambda") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "binder") == 0) {
                        node.payload.lambda.binder_id = (uint32_t)json_parse_uint64(ctx);
                    } else if (strcmp(pk, "captured_count") == 0) {
                        node.payload.lambda.captured_count = (uint32_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Effect") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "effect_tag") == 0) {
                        node.payload.effect_tag = (uint8_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Inject") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "tag_index") == 0) {
                        node.payload.tag_index = (uint16_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Project") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "field_index") == 0) {
                        node.payload.field_index = (uint16_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "Ref") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "fragment_id") == 0) {
                        node.payload.fragment_id = json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else if (strcmp(ptype, "LetRec") == 0) {
                json_expect(ctx, '{');
                while (json_peek(ctx) != '}' && json_peek(ctx) != -1) {
                    char pk[32];
                    json_parse_string(ctx, pk, sizeof(pk));
                    json_expect(ctx, ':');
                    if (strcmp(pk, "binder") == 0) {
                        node.payload.letrec.binder_id = (uint32_t)json_parse_uint64(ctx);
                    } else {
                        json_skip_value(ctx);
                    }
                    if (json_peek(ctx) == ',') json_expect(ctx, ',');
                }
                json_expect(ctx, '}');
            } else {
                /* Apply, Let, Tuple, etc. — no payload or we skip it */
                json_skip_value(ctx);
            }

            json_expect(ctx, '}'); /* close outer payload object */
        } else {
            json_skip_value(ctx);
        }

        if (json_peek(ctx) == ',') json_expect(ctx, ',');
    }

    return node;
}

/* -----------------------------------------------------------------------
 * Parse the top-level SemanticGraph JSON
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_load_json(const char *path) {
    FILE *fp = fopen(path, "rb");
    if (!fp) {
        fprintf(stderr, "error: cannot open '%s'\n", path);
        return NULL;
    }
    fseek(fp, 0, SEEK_END);
    long fsize = ftell(fp);
    fseek(fp, 0, SEEK_SET);

    char *data = malloc((size_t)fsize + 1);
    if (!data) {
        fprintf(stderr, "error: out of memory loading '%s'\n", path);
        fclose(fp);
        return NULL;
    }
    size_t nread = fread(data, 1, (size_t)fsize, fp);
    fclose(fp);
    data[nread] = '\0';

    json_ctx_t ctx = { .data = data, .pos = 0, .len = nread };
    iris_graph_t *g = iris_graph_raw_alloc();

    json_expect(&ctx, '{');

    char key[64];
    while (json_peek(&ctx) != '}' && json_peek(&ctx) != -1) {
        json_parse_string(&ctx, key, sizeof(key));
        json_expect(&ctx, ':');

        if (strcmp(key, "root") == 0) {
            g->root = json_parse_uint64(&ctx);
        } else if (strcmp(key, "nodes") == 0) {
            /* "nodes" is an object: {"id_str": {node...}, ...} */
            json_expect(&ctx, '{');
            while (json_peek(&ctx) != '}' && json_peek(&ctx) != -1) {
                json_skip_string(&ctx); /* key is the id as string */
                json_expect(&ctx, ':');
                json_expect(&ctx, '{');
                iris_node_t node = parse_node(&ctx);
                json_expect(&ctx, '}');
                iris_graph_raw_add_node(g, node);
                if (json_peek(&ctx) == ',') json_expect(&ctx, ',');
            }
            json_expect(&ctx, '}');
        } else if (strcmp(key, "edges") == 0) {
            /* "edges" is an array: [{source, target, port, label}, ...] */
            json_expect(&ctx, '[');
            while (json_peek(&ctx) != ']' && json_peek(&ctx) != -1) {
                iris_edge_t edge;
                memset(&edge, 0, sizeof(edge));
                json_expect(&ctx, '{');
                while (json_peek(&ctx) != '}' && json_peek(&ctx) != -1) {
                    char ek[32];
                    json_parse_string(&ctx, ek, sizeof(ek));
                    json_expect(&ctx, ':');
                    if (strcmp(ek, "source") == 0) {
                        edge.source = json_parse_uint64(&ctx);
                    } else if (strcmp(ek, "target") == 0) {
                        edge.target = json_parse_uint64(&ctx);
                    } else if (strcmp(ek, "port") == 0) {
                        edge.port = (uint8_t)json_parse_uint64(&ctx);
                    } else if (strcmp(ek, "label") == 0) {
                        char lbl[32];
                        json_parse_string(&ctx, lbl, sizeof(lbl));
                        edge.label = parse_edge_label(lbl);
                    } else {
                        json_skip_value(&ctx);
                    }
                    if (json_peek(&ctx) == ',') json_expect(&ctx, ',');
                }
                json_expect(&ctx, '}');
                iris_graph_raw_add_edge(g, edge);
                if (json_peek(&ctx) == ',') json_expect(&ctx, ',');
            }
            json_expect(&ctx, ']');
        } else {
            json_skip_value(&ctx);
        }

        if (json_peek(&ctx) == ',') json_expect(&ctx, ',');
    }

    free(data);

    fprintf(stderr, "loaded graph: %u nodes, %u edges, root=%lu\n",
            g->node_count, g->edge_count, (unsigned long)g->root);

    return g;
>>>>>>> worktree-agent-a01b95aa
}
