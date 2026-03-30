/* iris_rt.h — IRIS C runtime declarations.
 *
 * Generated C interpreters (#include "iris_rt.h") call these functions.
 * Implementations live in the corresponding iris_rt.c (not yet written).
 */

#ifndef IRIS_RT_H
#define IRIS_RT_H

#include <stdint.h>

/* Opaque value type used by all IRIS runtime functions. */
typedef struct iris_value iris_value_t;

/* -----------------------------------------------------------------------
 * Value constructors
 * ----------------------------------------------------------------------- */
iris_value_t *iris_int(int64_t v);
iris_value_t *iris_string(const char *s);
iris_value_t *iris_unit(void);

/* -----------------------------------------------------------------------
 * Truthiness (for Guard predicates)
 * ----------------------------------------------------------------------- */
int iris_is_truthy(iris_value_t *v);

/* -----------------------------------------------------------------------
 * Arithmetic (0x00-0x09)
 * ----------------------------------------------------------------------- */
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

/* -----------------------------------------------------------------------
 * Bitwise (0x10-0x15)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_bitand(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_bitor(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_bitxor(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_bitnot(iris_value_t *a);
iris_value_t *iris_shl(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_shr(iris_value_t *a, iris_value_t *b);

/* -----------------------------------------------------------------------
 * Comparison (0x20-0x25)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_eq(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_ne(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_lt(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_gt(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_le(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_ge(iris_value_t *a, iris_value_t *b);

/* -----------------------------------------------------------------------
 * Graph introspection (0x80-0x8F)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_self_graph(void);
iris_value_t *iris_graph_nodes(iris_value_t *g);
iris_value_t *iris_graph_get_kind(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_get_prim_op(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_set_prim_op(iris_value_t *g, iris_value_t *node, iris_value_t *op);
iris_value_t *iris_graph_add_node_rt(iris_value_t *g, iris_value_t *kind);
iris_value_t *iris_graph_connect(iris_value_t *g, iris_value_t *src, iris_value_t *tgt, iris_value_t *port);
iris_value_t *iris_graph_disconnect(iris_value_t *g, iris_value_t *src, iris_value_t *port);
iris_value_t *iris_graph_replace_subtree(iris_value_t *g, iris_value_t *old_node, iris_value_t *new_node);
iris_value_t *iris_graph_eval(iris_value_t *g, iris_value_t *inputs);
iris_value_t *iris_graph_get_root(iris_value_t *g);
iris_value_t *iris_graph_add_guard_rt(iris_value_t *g, iris_value_t *pred, iris_value_t *body, iris_value_t *fallback);
iris_value_t *iris_graph_add_ref_rt(iris_value_t *g, iris_value_t *frag);
iris_value_t *iris_graph_set_cost(iris_value_t *g, iris_value_t *node, iris_value_t *cost);
iris_value_t *iris_graph_get_lit_value(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_outgoing(iris_value_t *g, iris_value_t *node);

/* -----------------------------------------------------------------------
 * Graph mutation (0xED-0xF1)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_graph_new(void);
iris_value_t *iris_graph_set_root(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_set_lit_value(iris_value_t *g, iris_value_t *node, iris_value_t *tag, iris_value_t *val);
iris_value_t *iris_graph_set_field_index(iris_value_t *g, iris_value_t *node, iris_value_t *idx);

/* -----------------------------------------------------------------------
 * Graph extended (0x60-0x66, 0x96-0x9F)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_graph_get_node_cost(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_set_node_type(iris_value_t *g, iris_value_t *node, iris_value_t *ty);
iris_value_t *iris_graph_get_node_type(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_edges(iris_value_t *g);
iris_value_t *iris_graph_get_arity(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_get_depth(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_get_lit_type_tag(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_edge_count(iris_value_t *g);
iris_value_t *iris_graph_edge_target(iris_value_t *g, iris_value_t *src, iris_value_t *port, iris_value_t *label);
iris_value_t *iris_graph_get_binder(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_eval_env(iris_value_t *g, iris_value_t *node, iris_value_t *env);
iris_value_t *iris_graph_get_tag(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_get_field_index(iris_value_t *g, iris_value_t *node);
iris_value_t *iris_graph_get_effect_tag(iris_value_t *g, iris_value_t *node);

/* -----------------------------------------------------------------------
 * Value introspection (0x9C-0x9E)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_value_get_tag(iris_value_t *v);
iris_value_t *iris_value_get_payload(iris_value_t *v);
iris_value_t *iris_value_make_tagged(iris_value_t *tag, iris_value_t *payload);

/* -----------------------------------------------------------------------
 * List / Tuple (0xC1-0xD6, 0xF0)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_list_append(iris_value_t *lst, iris_value_t *elem);
iris_value_t *iris_list_nth(iris_value_t *lst, iris_value_t *idx);
iris_value_t *iris_list_take(iris_value_t *lst, iris_value_t *n);
iris_value_t *iris_list_drop(iris_value_t *lst, iris_value_t *n);
iris_value_t *iris_list_sort(iris_value_t *lst);
iris_value_t *iris_list_dedup(iris_value_t *lst);
iris_value_t *iris_list_range(iris_value_t *lo, iris_value_t *hi);
iris_value_t *iris_list_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_list_len(iris_value_t *lst);
iris_value_t *iris_sort_by(iris_value_t *lst, iris_value_t *cmp);
iris_value_t *iris_tuple_get(iris_value_t *tup, iris_value_t *idx);
iris_value_t *iris_tuple_len(iris_value_t *tup);

/* -----------------------------------------------------------------------
 * Map (0xC8-0xCD)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_map_insert(iris_value_t *m, iris_value_t *k, iris_value_t *v);
iris_value_t *iris_map_get(iris_value_t *m, iris_value_t *k);
iris_value_t *iris_map_remove(iris_value_t *m, iris_value_t *k);
iris_value_t *iris_map_keys(iris_value_t *m);
iris_value_t *iris_map_values(iris_value_t *m);
iris_value_t *iris_map_size(iris_value_t *m);

/* -----------------------------------------------------------------------
 * String (0xB0-0xC0)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_str_len(iris_value_t *s);
iris_value_t *iris_str_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_str_slice(iris_value_t *s, iris_value_t *lo, iris_value_t *hi);
iris_value_t *iris_str_contains(iris_value_t *s, iris_value_t *sub);
iris_value_t *iris_str_split(iris_value_t *s, iris_value_t *sep);
iris_value_t *iris_str_join(iris_value_t *lst, iris_value_t *sep);
iris_value_t *iris_str_to_int(iris_value_t *s);
iris_value_t *iris_int_to_string(iris_value_t *v);
iris_value_t *iris_str_eq(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_str_starts_with(iris_value_t *s, iris_value_t *prefix);
iris_value_t *iris_str_ends_with(iris_value_t *s, iris_value_t *suffix);
iris_value_t *iris_str_replace(iris_value_t *s, iris_value_t *old, iris_value_t *new_str);
iris_value_t *iris_str_trim(iris_value_t *s);
iris_value_t *iris_str_upper(iris_value_t *s);
iris_value_t *iris_str_lower(iris_value_t *s);
iris_value_t *iris_str_chars(iris_value_t *s);
iris_value_t *iris_char_at(iris_value_t *s, iris_value_t *idx);

/* -----------------------------------------------------------------------
 * String builder (0xD3-0xD5)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_buf_new(void);
iris_value_t *iris_buf_push(iris_value_t *buf, iris_value_t *s);
iris_value_t *iris_buf_finish(iris_value_t *buf);

/* -----------------------------------------------------------------------
 * Higher-order (0x30-0x32)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_map(iris_value_t *f, iris_value_t *lst);
iris_value_t *iris_filter(iris_value_t *f, iris_value_t *lst);
iris_value_t *iris_zip(iris_value_t *a, iris_value_t *b);

/* -----------------------------------------------------------------------
 * Conversion (0x40-0x44)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_int_to_float(iris_value_t *v);
iris_value_t *iris_float_to_int(iris_value_t *v);
iris_value_t *iris_float_to_bits(iris_value_t *v);
iris_value_t *iris_bits_to_float(iris_value_t *v);
iris_value_t *iris_bool_to_int(iris_value_t *v);

/* -----------------------------------------------------------------------
 * Math (0xD8-0xE3)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_math_sqrt(iris_value_t *v);
iris_value_t *iris_math_log(iris_value_t *v);
iris_value_t *iris_math_exp(iris_value_t *v);
iris_value_t *iris_math_sin(iris_value_t *v);
iris_value_t *iris_math_cos(iris_value_t *v);
iris_value_t *iris_math_floor(iris_value_t *v);
iris_value_t *iris_math_ceil(iris_value_t *v);
iris_value_t *iris_math_round(iris_value_t *v);
iris_value_t *iris_math_pi(void);
iris_value_t *iris_math_e(void);
iris_value_t *iris_random_int(iris_value_t *lo, iris_value_t *hi);
iris_value_t *iris_random_float(void);

/* -----------------------------------------------------------------------
 * Bytes (0xE6-0xE8)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_bytes_from_ints(iris_value_t *lst);
iris_value_t *iris_bytes_concat(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_bytes_len(iris_value_t *b);

/* -----------------------------------------------------------------------
 * Lazy (0xE9-0xEC)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_lazy_unfold(iris_value_t *seed, iris_value_t *f);
iris_value_t *iris_thunk_force(iris_value_t *thunk);
iris_value_t *iris_lazy_take(iris_value_t *n, iris_value_t *lst);
iris_value_t *iris_lazy_map(iris_value_t *f, iris_value_t *lst);

/* -----------------------------------------------------------------------
 * State (0x50-0x55)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_state_empty(void);
iris_value_t *iris_state_get(iris_value_t *st, iris_value_t *key);
iris_value_t *iris_state_set(iris_value_t *st, iris_value_t *key, iris_value_t *val);

/* -----------------------------------------------------------------------
 * Parallel (0x90-0x95)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_par_eval(iris_value_t *lst);
iris_value_t *iris_par_map(iris_value_t *f, iris_value_t *lst);
iris_value_t *iris_par_fold(iris_value_t *f, iris_value_t *init, iris_value_t *lst);
iris_value_t *iris_spawn(iris_value_t *f);
iris_value_t *iris_await_future(iris_value_t *fut);
iris_value_t *iris_par_zip_with(iris_value_t *f, iris_value_t *a, iris_value_t *b);

/* -----------------------------------------------------------------------
 * Effects / evolve (0xA0-0xA3)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_evolve_subprogram(iris_value_t *g, iris_value_t *fitness, iris_value_t *iters);
iris_value_t *iris_perform_effect(iris_value_t *tag, iris_value_t *payload);
iris_value_t *iris_graph_eval_ref(iris_value_t *g, iris_value_t *ref, iris_value_t *inputs);
iris_value_t *iris_compile_source_json(iris_value_t *json);

/* -----------------------------------------------------------------------
 * Knowledge graph (0x70-0x7B)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_kg_empty(void);
iris_value_t *iris_kg_add_node(iris_value_t *kg, iris_value_t *id, iris_value_t *data);
iris_value_t *iris_kg_add_edge(iris_value_t *kg, iris_value_t *src, iris_value_t *tgt, iris_value_t *ty, iris_value_t *w);
iris_value_t *iris_kg_get_node(iris_value_t *kg, iris_value_t *id);
iris_value_t *iris_kg_neighbors(iris_value_t *kg, iris_value_t *id, iris_value_t *ty);
iris_value_t *iris_kg_set_edge_weight(iris_value_t *kg, iris_value_t *src, iris_value_t *tgt, iris_value_t *w);
iris_value_t *iris_kg_map_nodes(iris_value_t *kg, iris_value_t *f);
iris_value_t *iris_kg_merge(iris_value_t *a, iris_value_t *b);
iris_value_t *iris_kg_query_by_edge_type(iris_value_t *kg, iris_value_t *ty);
iris_value_t *iris_kg_node_count(iris_value_t *kg);
iris_value_t *iris_kg_edge_count(iris_value_t *kg);

/* -----------------------------------------------------------------------
 * I/O substrate (0xF2-0xF8)
 * ----------------------------------------------------------------------- */
iris_value_t *iris_file_read(iris_value_t *path);
iris_value_t *iris_compile_source(iris_value_t *src);
iris_value_t *iris_debug_print(iris_value_t *v);
iris_value_t *iris_module_eval(iris_value_t *mod, iris_value_t *name, iris_value_t *args);
iris_value_t *iris_compile_test_file(iris_value_t *path, iris_value_t *root);
iris_value_t *iris_module_test_count(iris_value_t *mod);
iris_value_t *iris_module_eval_test(iris_value_t *mod, iris_value_t *idx);

#endif /* IRIS_RT_H */
