/*
<<<<<<< HEAD
 * iris_graph.h — SemanticGraph data structures for the IRIS C runtime
 *
 * Mirrors the Rust SemanticGraph (iris-types/src/graph.rs): nodes with
 * 20 possible kinds, directed edges with (source, target, port, label).
 * Graph data is refcounted for COW semantics (iris_graph_set_root creates
 * a new wrapper without copying the underlying data).
 */

=======
 * iris_graph.h -- Low-level graph data structures for the IRIS C bootstrap.
 *
 * These are internal structs. The public API uses iris_value_t* wrappers
 * declared in iris_rt.h.
 */
>>>>>>> worktree-agent-a01b95aa
#ifndef IRIS_GRAPH_H
#define IRIS_GRAPH_H

#include <stdint.h>
#include <stddef.h>
<<<<<<< HEAD
#include "iris_rt.h"

/* -----------------------------------------------------------------------
 * Node kinds — matches NodeKind repr(u8) in graph.rs
 * ----------------------------------------------------------------------- */

#define NK_PRIM      0x00
#define NK_APPLY     0x01
#define NK_LAMBDA    0x02
#define NK_LET       0x03
#define NK_MATCH     0x04
#define NK_LIT       0x05
#define NK_REF       0x06
#define NK_NEURAL    0x07
#define NK_FOLD      0x08
#define NK_UNFOLD    0x09
#define NK_EFFECT    0x0A
#define NK_TUPLE     0x0B
#define NK_INJECT    0x0C
#define NK_PROJECT   0x0D
#define NK_TYPEABST  0x0E
#define NK_TYPEAPP   0x0F
#define NK_LETREC    0x10
#define NK_GUARD     0x11
#define NK_REWRITE   0x12
#define NK_EXTERN    0x13

/* -----------------------------------------------------------------------
 * Edge labels — matches EdgeLabel repr(u8)
=======

/* -----------------------------------------------------------------------
 * NodeKind (matches Rust's NodeKind repr(u8))
 * ----------------------------------------------------------------------- */

#define NK_PRIM       0x00
#define NK_APPLY      0x01
#define NK_LAMBDA     0x02
#define NK_LET        0x03
#define NK_MATCH      0x04
#define NK_LIT        0x05
#define NK_REF        0x06
#define NK_NEURAL     0x07
#define NK_FOLD       0x08
#define NK_UNFOLD     0x09
#define NK_EFFECT     0x0A
#define NK_TUPLE      0x0B
#define NK_INJECT     0x0C
#define NK_PROJECT    0x0D
#define NK_TYPE_ABST  0x0E
#define NK_TYPE_APP   0x0F
#define NK_LET_REC    0x10
#define NK_GUARD      0x11
#define NK_REWRITE    0x12
#define NK_EXTERN     0x13

/* -----------------------------------------------------------------------
 * EdgeLabel (matches Rust's EdgeLabel repr)
>>>>>>> worktree-agent-a01b95aa
 * ----------------------------------------------------------------------- */

#define EL_ARGUMENT     0
#define EL_SCRUTINEE    1
#define EL_BINDING      2
#define EL_CONTINUATION 3
#define EL_DECREASE     4

/* -----------------------------------------------------------------------
<<<<<<< HEAD
 * Node
 * ----------------------------------------------------------------------- */

typedef struct iris_node {
    uint64_t id;
    uint8_t  kind;          /* NK_* constant (0-0x13) */
    uint8_t  arity;
    union {
        /* Prim */
        uint8_t prim_opcode;
        /* Lambda */
        struct { uint32_t binder_id; uint32_t captured_count; } lambda;
        /* Lit */
        struct {
            uint8_t   type_tag;
            uint8_t  *value;
            size_t    value_len;
        } lit;
        /* Guard */
        struct { uint64_t pred; uint64_t body; uint64_t fallback; } guard;
        /* Inject */
        struct { uint16_t tag_index; } inject;
        /* Project */
        struct { uint16_t field_index; } project;
        /* Effect */
        uint8_t effect_tag;
        /* LetRec */
        struct { uint32_t binder_id; } letrec;
        /* Match */
        struct { uint16_t arm_count; } match_info;
=======
 * Node payload
 * ----------------------------------------------------------------------- */

typedef struct {
    uint8_t type_tag;
    uint8_t *value;     /* raw bytes (little-endian) */
    uint32_t value_len;
} iris_lit_payload_t;

typedef struct {
    uint32_t binder_id;
    uint32_t captured_count;
} iris_lambda_payload_t;

typedef struct {
    uint64_t predicate_node;
    uint64_t body_node;
    uint64_t fallback_node;
} iris_guard_payload_t;

typedef struct iris_node {
    uint64_t id;
    uint8_t  kind;          /* NK_* constant */
    uint8_t  arity;
    uint64_t type_sig;
    uint64_t salt;

    /* Payload — which field is valid depends on `kind` */
    union {
        uint8_t                prim_opcode;     /* NK_PRIM */
        iris_lit_payload_t     lit;             /* NK_LIT */
        iris_lambda_payload_t  lambda;          /* NK_LAMBDA */
        iris_guard_payload_t   guard;           /* NK_GUARD */
        uint8_t                effect_tag;      /* NK_EFFECT */
        uint16_t               tag_index;       /* NK_INJECT */
        uint16_t               field_index;     /* NK_PROJECT */
        uint64_t               fragment_id;     /* NK_REF */
        struct {                                /* NK_LET_REC */
            uint32_t binder_id;
        } letrec;
        struct {                                /* NK_FOLD */
            uint8_t *descriptor;
            uint32_t descriptor_len;
        } fold;
>>>>>>> worktree-agent-a01b95aa
    } payload;
} iris_node_t;

/* -----------------------------------------------------------------------
 * Edge
 * ----------------------------------------------------------------------- */

<<<<<<< HEAD
typedef struct iris_edge {
    uint64_t source;
    uint64_t target;
    uint8_t  port;
    uint8_t  label;          /* EL_* constant */
} iris_edge_t;

/* -----------------------------------------------------------------------
 * Graph data (refcounted, shared between COW wrappers)
 * ----------------------------------------------------------------------- */

typedef struct iris_graph_data {
    uint32_t     refcount;
    iris_node_t *nodes;
    size_t       node_count;
    size_t       node_capacity;
    iris_edge_t *edges;
    size_t       edge_count;
    size_t       edge_capacity;
} iris_graph_data_t;

/* -----------------------------------------------------------------------
 * Graph wrapper (root + shared data)
 * ----------------------------------------------------------------------- */

struct iris_graph {
    uint32_t           refcount;    /* wrapper refcount */
    uint64_t           root;
    iris_graph_data_t *data;
};

/* -----------------------------------------------------------------------
 * Graph lifecycle
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_new(void);
void          iris_graph_retain(iris_graph_t *g);
void          iris_graph_release(iris_graph_t *g);

/* -----------------------------------------------------------------------
 * Node/edge queries
 * ----------------------------------------------------------------------- */

iris_node_t *iris_graph_find_node(iris_graph_t *g, uint64_t id);
size_t       iris_graph_find_edges(iris_graph_t *g, uint64_t source,
                                   iris_edge_t **out);
uint64_t     iris_graph_edge_target(iris_graph_t *g, uint64_t source,
                                    uint8_t port, uint8_t label);
size_t       iris_graph_node_count(iris_graph_t *g);
size_t       iris_graph_edge_count(iris_graph_t *g);

/* -----------------------------------------------------------------------
 * Graph construction (used by JSON loader)
 * ----------------------------------------------------------------------- */

void iris_graph_add_node(iris_graph_t *g, iris_node_t node);
void iris_graph_add_edge(iris_graph_t *g, iris_edge_t edge);

/* -----------------------------------------------------------------------
 * COW: create new wrapper with different root, sharing data
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_set_root(iris_graph_t *g, uint64_t new_root);

/* -----------------------------------------------------------------------
 * Argument collection: find all Argument edges from source, sorted by port
 * ----------------------------------------------------------------------- */

size_t iris_graph_argument_targets(iris_graph_t *g, uint64_t source,
                                   uint64_t *out, size_t max);
=======
typedef struct {
    uint64_t source;
    uint64_t target;
    uint8_t  port;
    uint8_t  label;     /* EL_* constant */
} iris_edge_t;

/* -----------------------------------------------------------------------
 * Graph
 * ----------------------------------------------------------------------- */

typedef struct iris_graph {
    uint64_t      root;
    iris_node_t  *nodes;
    uint32_t      node_count;
    uint32_t      node_cap;
    iris_edge_t  *edges;
    uint32_t      edge_count;
    uint32_t      edge_cap;
} iris_graph_t;

/* -----------------------------------------------------------------------
 * Low-level graph accessors (internal, prefixed iris_graph_raw_*)
 * ----------------------------------------------------------------------- */

iris_graph_t *iris_graph_raw_alloc(void);
void          iris_graph_raw_free(iris_graph_t *g);
iris_graph_t *iris_graph_raw_clone(const iris_graph_t *g);
iris_node_t  *iris_graph_raw_find_node(const iris_graph_t *g, uint64_t id);
uint32_t      iris_graph_raw_outgoing(const iris_graph_t *g, uint64_t source,
                                      uint8_t label, uint64_t *out, uint32_t out_cap);
int           iris_graph_raw_edge_target(const iris_graph_t *g, uint64_t source,
                                         uint8_t port, uint8_t label,
                                         uint64_t *out_target);

/* Node insertion (used by JSON loader) */
void iris_graph_raw_add_node(iris_graph_t *g, iris_node_t node);
void iris_graph_raw_add_edge(iris_graph_t *g, iris_edge_t edge);
>>>>>>> worktree-agent-a01b95aa

#endif /* IRIS_GRAPH_H */
