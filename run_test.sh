#!/bin/bash

# Ensure a test file was provided
if [ -z "$1" ]; then
    echo "Usage: ./run_test.sh <path/to/test.ts>"
    echo "Example: ./run_test.sh tests/closure_basic.ts"
    exit 1
fi

TEST_FILE=$1
# Extract the base name without the path and without the .ts extension
BASE_NAME=$(basename "$TEST_FILE" .ts)

OUT_DIR="build"
mkdir -p "$OUT_DIR"
OUT_PATH="$OUT_DIR/$BASE_NAME"

echo "Building ts-rt-stubs in release mode..."
cargo build --release -p ts-rt-stubs
mkdir -p lib
cp target/release/libts_rt_stubs.a lib/libts_rt_stubs.a

echo "Compiling $TEST_FILE to $OUT_PATH..."
LIBRARY_PATH=$(pwd)/lib \
RUSTFLAGS="-L native=$(pwd)/lib" \
LLVM_SYS_221_PREFIX=/usr/lib/llvm-22 \
cargo run -q -p compiler-core -- --no-cache "$TEST_FILE" -o "$OUT_PATH"

if [ $? -eq 0 ]; then
    echo "Compilation successful! Running ./$OUT_PATH..."
    echo "--------------------------------------------------"
    ./"$OUT_PATH"
    echo "--------------------------------------------------"
else
    echo "Compilation failed."
    exit 1
fi
