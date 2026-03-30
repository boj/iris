/*
 * iris_json_write.c -- Serialize iris_graph_t to the serde JSON format.
 *
 * Produces JSON compatible with iris_graph_load_json (round-trip).
 * Output goes to a FILE*.
 */

#include "iris_rt.h"
#include "iris_graph.h"

#include <stdio.h>
#include <string.h>
#include <inttypes.h>

static const char *kind_to_string(uint8_t kind) {
    switch (kind) {
    case NK_PRIM:      return "Prim";
    case NK_APPLY:     return "Apply";
    case NK_LAMBDA:    return "Lambda";
    case NK_LET:       return "Let";
    case NK_MATCH:     return "Match";
    case NK_LIT:       return "Lit";
    case NK_REF:       return "Ref";
    case NK_NEURAL:    return "Neural";
    case NK_FOLD:      return "Fold";
    case NK_UNFOLD:    return "Unfold";
    case NK_EFFECT:    return "Effect";
    case NK_TUPLE:     return "Tuple";
    case NK_INJECT:    return "Inject";
    case NK_PROJECT:   return "Project";
    case NK_TYPE_ABST: return "TypeAbst";
    case NK_TYPE_APP:  return "TypeApp";
    case NK_LET_REC:   return "LetRec";
    case NK_GUARD:     return "Guard";
    case NK_REWRITE:   return "Rewrite";
    case NK_EXTERN:    return "Extern";
    default:           return "Unknown";
    }
}

static const char *label_to_string(uint8_t label) {
    switch (label) {
    case EL_ARGUMENT:     return "Argument";
    case EL_SCRUTINEE:    return "Scrutinee";
    case EL_BINDING:      return "Binding";
    case EL_CONTINUATION: return "Continuation";
    case EL_DECREASE:     return "Decrease";
    default:              return "Argument";
    }
}

static void write_payload(FILE *fp, const iris_node_t *n) {
    switch (n->kind) {
    case NK_PRIM:
        fprintf(fp, "{\"Prim\":{\"opcode\":%u}}", n->payload.prim_opcode);
        break;
    case NK_LIT:
        fprintf(fp, "{\"Lit\":{\"type_tag\":%u,\"value\":[", n->payload.lit.type_tag);
        for (uint32_t i = 0; i < n->payload.lit.value_len; i++) {
            if (i > 0) fprintf(fp, ",");
            fprintf(fp, "%u", n->payload.lit.value[i]);
        }
        fprintf(fp, "]}}");
        break;
    case NK_GUARD:
        fprintf(fp, "{\"Guard\":{\"predicate_node\":%" PRIu64
                ",\"body_node\":%" PRIu64 ",\"fallback_node\":%" PRIu64 "}}",
                n->payload.guard.predicate_node,
                n->payload.guard.body_node,
                n->payload.guard.fallback_node);
        break;
    case NK_LAMBDA:
        fprintf(fp, "{\"Lambda\":{\"binder_id\":%u,\"captured_count\":%u}}",
                n->payload.lambda.binder_id, n->payload.lambda.captured_count);
        break;
    case NK_INJECT:
        fprintf(fp, "{\"Inject\":{\"tag_index\":%u}}", n->payload.tag_index);
        break;
    case NK_PROJECT:
        fprintf(fp, "{\"Project\":{\"field_index\":%u}}", n->payload.field_index);
        break;
    case NK_EFFECT:
        fprintf(fp, "{\"Effect\":{\"effect_tag\":%u}}", n->payload.effect_tag);
        break;
    case NK_REF:
        fprintf(fp, "{\"Ref\":{\"fragment_id\":%" PRIu64 "}}", n->payload.fragment_id);
        break;
    case NK_LET_REC:
        fprintf(fp, "{\"LetRec\":{\"binder_id\":%u}}", n->payload.letrec.binder_id);
        break;
    default:
        /* Kinds with no payload: Tuple, Apply, Let, Match, Fold, Unfold,
           TypeAbst, TypeApp, Rewrite, Extern, Neural */
        fprintf(fp, "\"%s\"", kind_to_string(n->kind));
        break;
    }
}

void iris_graph_write_json(FILE *fp, const iris_graph_t *g) {
    fprintf(fp, "{\n  \"root\": %" PRIu64 ",\n", g->root);

    /* Nodes */
    fprintf(fp, "  \"nodes\": {\n");
    for (uint32_t i = 0; i < g->node_count; i++) {
        const iris_node_t *n = &g->nodes[i];
        if (i > 0) fprintf(fp, ",\n");
        fprintf(fp, "    \"%" PRIu64 "\": {"
                "\"id\":%" PRIu64 ","
                "\"kind\":\"%s\","
                "\"type_sig\":%" PRIu64 ","
                "\"cost\":\"Unit\","
                "\"arity\":%u,"
                "\"resolution_depth\":0,"
                "\"salt\":%" PRIu64 ","
                "\"payload\":",
                n->id, n->id, kind_to_string(n->kind),
                n->type_sig, n->arity, n->salt);
        write_payload(fp, n);
        fprintf(fp, "}");
    }
    fprintf(fp, "\n  },\n");

    /* Edges — skip synthesized Guard edges (they have Guard-node sources) */
    fprintf(fp, "  \"edges\": [\n");
    int first_edge = 1;
    for (uint32_t i = 0; i < g->edge_count; i++) {
        const iris_edge_t *e = &g->edges[i];
        /* Skip synthesized Guard edges — they'll be re-synthesized on load */
        iris_node_t *src = iris_graph_raw_find_node((iris_graph_t*)g, e->source);
        if (src && src->kind == NK_GUARD && e->label == EL_ARGUMENT) continue;

        if (!first_edge) fprintf(fp, ",\n");
        first_edge = 0;
        fprintf(fp, "    {\"source\":%" PRIu64 ",\"target\":%" PRIu64
                ",\"port\":%u,\"label\":\"%s\"}",
                e->source, e->target, e->port, label_to_string(e->label));
    }
    fprintf(fp, "\n  ],\n");

    /* type_env, cost, resolution, hash — minimal valid values */
    fprintf(fp, "  \"type_env\": {\"types\": {}},\n");
    fprintf(fp, "  \"cost\": \"Unknown\",\n");
    fprintf(fp, "  \"resolution\": \"Implementation\",\n");
    fprintf(fp, "  \"hash\": [0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]\n");
    fprintf(fp, "}\n");
}
