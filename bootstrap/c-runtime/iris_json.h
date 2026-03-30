/*
 * iris_json.h — Minimal JSON parser and graph loader
 */

#ifndef IRIS_JSON_H
#define IRIS_JSON_H

#include "iris_graph.h"

/* Load a SemanticGraph from a JSON file (interpreter.json format). */
iris_graph_t *iris_load_json(const char *path);

#endif /* IRIS_JSON_H */
