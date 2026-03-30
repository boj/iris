CC      ?= gcc
CFLAGS  := -O2 -Wall -Wextra -Wno-unused-parameter -std=c11
LDFLAGS := -lm

SRCS := main.c iris_graph.c iris_prims.c iris_eval.c iris_interp_compiled.c
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
