<<<<<<< HEAD
CC ?= gcc
CFLAGS = -O2 -Wall -Wextra -Wno-unused-parameter -std=c11 -D_POSIX_C_SOURCE=199309L
LDFLAGS = -lm

SRCS = iris_rt.c iris_graph.c iris_json.c iris_eval.c iris_prims.c iris_effects.c iris_main.c
OBJS = $(SRCS:.c=.o)

all: iris-stage0

iris-stage0: $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

# Dependencies
iris_rt.o: iris_rt.c iris_rt.h iris_graph.h
iris_graph.o: iris_graph.c iris_graph.h iris_rt.h
iris_json.o: iris_json.c iris_json.h iris_graph.h iris_rt.h
iris_eval.o: iris_eval.c iris_eval.h iris_rt.h iris_graph.h iris_json.h
iris_prims.o: iris_prims.c iris_eval.h iris_rt.h iris_graph.h iris_json.h
iris_effects.o: iris_effects.c iris_eval.h iris_rt.h iris_graph.h
iris_main.o: iris_main.c iris_rt.h iris_graph.h iris_json.h iris_eval.h

clean:
	rm -f $(OBJS) iris-stage0

.PHONY: all clean
=======
# Makefile for iris-stage0-c — the C bootstrap evaluator.
#
# Usage:
#   nix-shell -p gcc --run 'cd bootstrap/c-runtime && make'
#
# On NixOS without gcc on PATH:
#   nix-shell -p gcc gnumake --run 'make -C bootstrap/c-runtime'

CC      ?= gcc
CFLAGS  := -O2 -Wall -Wextra -Wno-unused-parameter -std=c11
LDFLAGS := -lm

SRCS := main.c iris_graph.c iris_prims.c iris_eval.c
OBJS := $(SRCS:.c=.o)
BIN  := iris-stage0-c

.PHONY: all clean

all: $(BIN)

$(BIN): $(OBJS)
	$(CC) $(CFLAGS) -o $@ $^ $(LDFLAGS)

%.o: %.c iris_rt.h iris_graph.h
	$(CC) $(CFLAGS) -c -o $@ $<

clean:
	rm -f $(OBJS) $(BIN)
>>>>>>> worktree-agent-a01b95aa
