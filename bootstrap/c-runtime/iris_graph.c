/*
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
    }
    return NULL;
}

/* -----------------------------------------------------------------------
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
}
