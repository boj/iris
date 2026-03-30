/*
 * main.c -- Entry point for iris-stage0-c.
 *
 * Usage:
 *   iris-stage0-c direct <program.json> [args...]
 *
 * Loads a JSON-compiled SemanticGraph, evaluates it, and prints the result.
 */

#include "iris_rt.h"
#include "iris_graph.h"

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static void print_value(iris_value_t *v, int depth) {
    if (!v) { printf("null"); return; }
    if (depth > 10) { printf("..."); return; }

    switch (v->tag) {
    case IRIS_INT:
        printf("%ld", (long)v->i);
        break;
    case IRIS_FLOAT64:
        printf("%g", v->f);
        break;
    case IRIS_BOOL:
        printf("%s", v->b ? "true" : "false");
        break;
    case IRIS_UNIT:
        printf("()");
        break;
    case IRIS_TUPLE:
        printf("(");
        for (uint32_t i = 0; i < v->tuple.len; i++) {
            if (i > 0) printf(", ");
            print_value(v->tuple.elems[i], depth + 1);
        }
        printf(")");
        break;
    case IRIS_PROGRAM:
        printf("<program:%u nodes>", v->graph ? v->graph->node_count : 0);
        break;
    case IRIS_TAGGED:
        printf("Tagged(%u, ", v->tagged.tag_index);
        print_value(v->tagged.payload, depth + 1);
        printf(")");
        break;
    case IRIS_STRING:
        printf("\"%s\"", v->str.data ? v->str.data : "");
        break;
    case IRIS_BYTES:
        printf("<bytes:%u>", v->bytes.len);
        break;
    default:
        printf("<unknown:%d>", v->tag);
        break;
    }
}

static iris_value_t *parse_arg(const char *s) {
    /* Try integer */
    char *end;
    long val = strtol(s, &end, 10);
    if (*end == '\0') {
        return iris_int(val);
    }
    /* Try float */
    double fval = strtod(s, &end);
    if (*end == '\0') {
        return iris_float64(fval);
    }
    /* Boolean */
    if (strcmp(s, "true") == 0) return iris_bool(true);
    if (strcmp(s, "false") == 0) return iris_bool(false);
    /* String */
    return iris_string(s, (uint32_t)strlen(s));
}

int main(int argc, char **argv) {
    if (argc < 3) {
        fprintf(stderr, "Usage: %s direct <program.json> [args...]\n", argv[0]);
        fprintf(stderr, "\nEvaluates a JSON-compiled IRIS SemanticGraph.\n");
        return 1;
    }

    const char *cmd = argv[1];

    /* compile: .iris source → JSON graph (stdout) */
    if (strcmp(cmd, "compile") == 0) {
        if (argc < 3) {
            fprintf(stderr, "Usage: %s compile <source.iris>\n", argv[0]);
            return 1;
        }
        /* Read source file */
        FILE *fp = fopen(argv[2], "r");
        if (!fp) { fprintf(stderr, "error: cannot open '%s'\n", argv[2]); return 1; }
        fseek(fp, 0, SEEK_END);
        long fsize = ftell(fp);
        fseek(fp, 0, SEEK_SET);
        char *source = malloc((size_t)fsize + 1);
        fread(source, 1, (size_t)fsize, fp);
        source[fsize] = '\0';
        fclose(fp);
        iris_value_t *src_val = iris_string(source, (uint32_t)fsize);
        free(source);

        /* Load and run tokenizer */
        iris_graph_t *tok = iris_graph_load_json("bootstrap/tokenizer.json");
        if (!tok) { fprintf(stderr, "error: cannot load tokenizer.json\n"); return 1; }
        iris_value_t *tokens = iris_eval_graph(tok, src_val);

        /* Load and run parser */
        iris_graph_t *par = iris_graph_load_json("bootstrap/parser.json");
        if (!par) { fprintf(stderr, "error: cannot load parser.json\n"); return 1; }
        iris_value_t *ast = iris_eval_graph(par, tokens);

        /* Load and run lowerer */
        iris_graph_t *low = iris_graph_load_json("bootstrap/lowerer.json");
        if (!low) { fprintf(stderr, "error: cannot load lowerer.json\n"); return 1; }
        iris_value_t *graph_val = iris_eval_graph(low, ast);

        /* The lowerer returns (program, metadata). Extract the program. */
        iris_value_t *compiled = graph_val;
        if (compiled && compiled->tag == IRIS_TUPLE && compiled->tuple.len >= 1) {
            compiled = compiled->tuple.elems[0];
        }
        if (!compiled || compiled->tag != IRIS_PROGRAM || !compiled->graph) {
            fprintf(stderr, "error: lowerer did not produce a Program value\n");
            print_value(graph_val, 0);
            fprintf(stderr, "\n");
            return 1;
        }

        /* Serialize the compiled graph to JSON on stdout */
        extern void iris_graph_write_json(FILE *fp, const iris_graph_t *g);
        iris_graph_write_json(stdout, compiled->graph);
        return 0;
    }

    if (strcmp(cmd, "direct") != 0 && strcmp(cmd, "info") != 0) {
        fprintf(stderr, "error: unknown command '%s'\n", cmd);
        fprintf(stderr, "Commands: direct, compile\n");
        return 1;
    }

    const char *json_path = argv[2];

    /* Load graph from JSON */
    iris_graph_t *graph = iris_graph_load_json(json_path);
    if (!graph) {
        fprintf(stderr, "error: failed to load graph from '%s'\n", json_path);
        return 1;
    }

    /* Build input from remaining args */
    iris_value_t *input;
    int nargs = argc - 3;
    if (nargs == 0) {
        input = iris_unit();
    } else if (nargs == 1) {
        input = parse_arg(argv[3]);
    } else {
        iris_value_t **elems = malloc((size_t)nargs * sizeof(iris_value_t *));
        for (int i = 0; i < nargs; i++) {
            elems[i] = parse_arg(argv[3 + i]);
        }
        input = iris_tuple(elems, (uint32_t)nargs);
        free(elems);
    }

    /* Evaluate */
    iris_value_t *result = iris_eval_graph(graph, input);

    /* Print result */
    print_value(result, 0);
    printf("\n");

    return 0;
}
