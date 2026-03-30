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
