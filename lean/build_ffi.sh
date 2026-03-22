#!/usr/bin/env bash
# Build the IRIS Lean kernel as a static library for Rust FFI.
#
# This script:
# 1. Runs `lake build` to compile Lean → C
# 2. Compiles the C files with Lean's leanc (which sets up include paths)
# 3. Archives them into a static library
#
# The resulting libIrisKernel.a can be linked from Rust.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

# Find Lean prefix
LEAN_PREFIX="$(lean --print-prefix)"
LEAN_LIB="$LEAN_PREFIX/lib/lean"
LEAN_INCLUDE="$LEAN_PREFIX/include"

echo "=== IRIS Lean Kernel FFI Build ==="
echo "Lean prefix: $LEAN_PREFIX"

# Step 1: Build with Lake
echo "--- Step 1: lake build ---"
lake build IrisKernel

# Step 2: Compile C files to objects
echo "--- Step 2: Compile C → object files ---"
BUILD_DIR=".lake/build"
IR_DIR="$BUILD_DIR/ir"
OBJ_DIR="$BUILD_DIR/obj"
mkdir -p "$OBJ_DIR"

# Use leanc as the C compiler — it knows the right include paths and flags
for cfile in "$IR_DIR"/IrisKernel/*.c "$IR_DIR"/IrisKernel.c; do
    if [ -f "$cfile" ]; then
        basename="$(basename "$cfile" .c)"
        echo "  Compiling $basename.c"
        leanc -c "$cfile" -o "$OBJ_DIR/$basename.o" -O2
    fi
done

# Step 3: Archive into static library
echo "--- Step 3: Archive → libIrisKernel.a ---"
LIB_DIR="$BUILD_DIR/lib"
mkdir -p "$LIB_DIR"
ar rcs "$LIB_DIR/libIrisKernel.a" "$OBJ_DIR"/*.o

echo ""
echo "=== Build complete ==="
echo "Static library: $SCRIPT_DIR/$LIB_DIR/libIrisKernel.a"
echo ""
echo "To link from Rust, add to build.rs:"
echo "  println!(\"cargo:rustc-link-search=$SCRIPT_DIR/$LIB_DIR\");"
echo "  println!(\"cargo:rustc-link-lib=static=IrisKernel\");"
echo "  println!(\"cargo:rustc-link-search=$LEAN_LIB\");"
echo "  println!(\"cargo:rustc-link-lib=static=leanrt\");"
echo "  println!(\"cargo:rustc-link-lib=static=leancpp\");"
echo "  println!(\"cargo:rustc-link-lib=static=Init\");"
echo "  println!(\"cargo:rustc-link-lib=static=Std\");"
