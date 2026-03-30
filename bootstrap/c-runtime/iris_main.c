/*
 * iris_main.c — Entry point for the IRIS bootstrap C runtime
 *
 * Commands:
 *   iris-stage0 direct <graph.json> [arg1 arg2 ...]
 *     Load a SemanticGraph from JSON, evaluate with integer arguments.
 *
 *   iris-stage0 eval <graph.json> <input.json>
 *     Load graph, parse inputs from a JSON file, evaluate.
 *
 *   iris-stage0 info <graph.json>
 *     Print graph statistics (node count, edge count, root).
 */

#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

#include "iris_rt.h"
#include "iris_graph.h"
#include "iris_json.h"
#include "iris_eval.h"

static void usage(void) {
    fprintf(stderr,
        "Usage:\n"
        "  iris-stage0 direct <graph.json> [arg1 arg2 ...]\n"
        "  iris-stage0 eval <graph.json>\n"
        "  iris-stage0 info <graph.json>\n"
    );
}

/* Parse a command-line argument as an iris value.
 * Integers: plain numbers (42, -7)
 * Strings: anything else
 */
static iris_value_t *parse_arg(const char *s) {
    char *end;
    long long n = strtoll(s, &end, 10);
    if (*end == '\0' && end != s) {
        return iris_int((int64_t)n);
    }
    /* Check for float */
    double f = strtod(s, &end);
    if (*end == '\0' && end != s) {
        return iris_f64(f);
    }
    /* Check for bool */
    if (strcmp(s, "true") == 0) return iris_bool(1);
    if (strcmp(s, "false") == 0) return iris_bool(0);
    /* String */
    return iris_string(s);
}

static int cmd_direct(int argc, char **argv) {
    if (argc < 1) { usage(); return 1; }
    const char *graph_path = argv[0];

    iris_graph_t *graph = iris_load_json(graph_path);
    if (!graph) {
        fprintf(stderr, "Failed to load graph from '%s'\n", graph_path);
        return 1;
    }

    /* Build inputs tuple from remaining args */
    int nargs = argc - 1;
    iris_value_t *inputs;
    if (nargs > 0) {
        iris_value_t **elems = (iris_value_t **)malloc(
            sizeof(iris_value_t *) * (size_t)nargs);
        for (int i = 0; i < nargs; i++) {
            elems[i] = parse_arg(argv[1 + i]);
        }
        inputs = iris_tuple(elems, (size_t)nargs);
        for (int i = 0; i < nargs; i++) iris_release(elems[i]);
        free(elems);
    } else {
        inputs = iris_unit();
    }

    struct timespec t0, t1;
    clock_gettime(CLOCK_MONOTONIC, &t0);

    iris_value_t *result = iris_eval(graph, inputs);

    clock_gettime(CLOCK_MONOTONIC, &t1);
    double elapsed_ms = (t1.tv_sec - t0.tv_sec) * 1000.0 +
                        (t1.tv_nsec - t0.tv_nsec) / 1000000.0;

    iris_print_value(result);
    fprintf(stderr, "[%.1f ms]\n", elapsed_ms);

    iris_release(result);
    iris_release(inputs);
    iris_graph_release(graph);
    return 0;
}

static int cmd_info(int argc, char **argv) {
    if (argc < 1) { usage(); return 1; }
    const char *graph_path = argv[0];

    iris_graph_t *graph = iris_load_json(graph_path);
    if (!graph) {
        fprintf(stderr, "Failed to load graph from '%s'\n", graph_path);
        return 1;
    }

    printf("Graph: %s\n", graph_path);
    printf("  Root:  %lu\n", (unsigned long)graph->root);
    printf("  Nodes: %zu\n", iris_graph_node_count(graph));
    printf("  Edges: %zu\n", iris_graph_edge_count(graph));

    /* Count node kinds */
    int kind_counts[0x20] = {0};
    for (size_t i = 0; i < graph->data->node_count; i++) {
        uint8_t k = graph->data->nodes[i].kind;
        if (k < 0x20) kind_counts[k]++;
    }
    const char *kind_names[] = {
        "Prim", "Apply", "Lambda", "Let", "Match", "Lit", "Ref",
        "Neural", "Fold", "Unfold", "Effect", "Tuple", "Inject",
        "Project", "TypeAbst", "TypeApp", "LetRec", "Guard",
        "Rewrite", "Extern"
    };
    for (int k = 0; k < 0x14; k++) {
        if (kind_counts[k] > 0) {
            printf("  %s: %d\n", kind_names[k], kind_counts[k]);
        }
    }

    iris_graph_release(graph);
    return 0;
}

int main(int argc, char **argv) {
    srand((unsigned)time(NULL));

    if (argc < 3) {
        usage();
        return 1;
    }

    const char *cmd = argv[1];

    if (strcmp(cmd, "direct") == 0) {
        return cmd_direct(argc - 2, argv + 2);
    } else if (strcmp(cmd, "eval") == 0) {
        return cmd_direct(argc - 2, argv + 2);
    } else if (strcmp(cmd, "info") == 0) {
        return cmd_info(argc - 2, argv + 2);
    } else {
        fprintf(stderr, "Unknown command: %s\n", cmd);
        usage();
        return 1;
    }
}
