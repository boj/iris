# Makefile — iris-clcu: IRIS L3 Hardware Materialization Engine
#
# Builds the CLCU container interpreter, TLB data structures,
# and TSO synchronization as a static library.

CC       ?= gcc
AR       ?= ar
CFLAGS    = -O2 -Wall -Wextra -Wpedantic -std=c11
CFLAGS   += -D_GNU_SOURCE
CFLAGS   += -march=native
LDFLAGS   =

# AVX-512 flags — applied only if the compiler supports them.
# The code uses #ifdef __AVX512F__ guards internally with scalar fallbacks,
# so compilation succeeds regardless.
AVX512_FLAGS = -mavx512f -mavx512bw -mavx512dq

# Check if compiler supports AVX-512 flags (some systems/VMs don't)
AVX512_SUPPORTED := $(shell $(CC) -mavx512f -xc -c /dev/null -o /dev/null 2>/dev/null && echo yes || echo no)
ifeq ($(AVX512_SUPPORTED),yes)
    CFLAGS += $(AVX512_FLAGS)
endif

# SSE4.2 for hardware CRC32 in tlb_map
CFLAGS += -msse4.2

SRCDIR   = src
BUILDDIR = build

SRCS     = $(SRCDIR)/arena.c \
           $(SRCDIR)/interpreter.c \
           $(SRCDIR)/tlb_map.c \
           $(SRCDIR)/tso_ring.c

OBJS     = $(patsubst $(SRCDIR)/%.c,$(BUILDDIR)/%.o,$(SRCS))

LIB      = $(BUILDDIR)/libiris_clcu.a

.PHONY: all clean check

all: $(LIB)
	@echo "Build complete: $(LIB)"

$(BUILDDIR):
	mkdir -p $(BUILDDIR)

$(BUILDDIR)/%.o: $(SRCDIR)/%.c $(SRCDIR)/clcu.h | $(BUILDDIR)
	$(CC) $(CFLAGS) -c $< -o $@

$(LIB): $(OBJS)
	$(AR) rcs $@ $^

# Compilation check — verify all objects compile cleanly
check: $(OBJS)
	@echo "All source files compiled successfully."
	@echo "Objects:"
	@ls -la $(OBJS)

clean:
	rm -rf $(BUILDDIR)
