#!/bin/bash

TEST_FILE=""
EXTRA_FLAGS=""
OPTIMIZE=0
DEBUG_RC=0

for arg in "$@"; do
    if [[ "$arg" == "--optimize" ]]; then
        OPTIMIZE=1
        EXTRA_FLAGS="$EXTRA_FLAGS --opt-level 3"
    elif [[ "$arg" == "--trace-ownership" ]]; then
        export RUST_LOG="ownership_inference=debug"
    elif [[ "$arg" == "--debug" ]]; then
        export RUST_LOG="debug"
        EXTRA_FLAGS="$EXTRA_FLAGS --verify-memory"
        DEBUG_RC=1
    elif [[ "$arg" == -* ]]; then
        EXTRA_FLAGS="$EXTRA_FLAGS $arg"
    else
        TEST_FILE="$arg"
    fi
done

if [ "$OPTIMIZE" -eq 0 ]; then
    EXTRA_FLAGS="$EXTRA_FLAGS --opt-level 0"
fi

# Ensure a test file was provided
if [ -z "$TEST_FILE" ]; then
    echo "Usage: ./run_test.sh [options] <path/to/test.ts>"
    echo "Options: --verify-memory, --optimize, --trace-ownership, --debug"
    echo "Example: ./run_test.sh --debug tests/closure_basic.ts"
    exit 1
fi

# Extract the base name without the path and without the .ts extension
BASE_NAME=$(basename "$TEST_FILE" .ts)

OUT_DIR="build"
mkdir -p "$OUT_DIR"
OUT_PATH="$OUT_DIR/$BASE_NAME"

if [ "$DEBUG_RC" == "1" ]; then
    RT_STUBS_FEATURES="--features debug_rc"
else
    RT_STUBS_FEATURES=""
fi

if [ "$OPTIMIZE" -eq 1 ]; then
    echo "Building ts-rt-stubs in RELEASE mode..."
    cargo build -p ts-rt-stubs --release $RT_STUBS_FEATURES
else
    echo "Building ts-rt-stubs in DEBUG mode..."
    cargo build -p ts-rt-stubs $RT_STUBS_FEATURES
fi

echo "Compiling $TEST_FILE to $OUT_PATH..."
LIBRARY_PATH=$(pwd)/lib \
RUSTFLAGS="-L native=$(pwd)/lib" \
LLVM_SYS_221_PREFIX=/usr/lib/llvm-22 \
cargo run -q -p compiler-core -- --no-cache $EXTRA_FLAGS "$TEST_FILE" -o "$OUT_PATH"

if [ $? -eq 0 ]; then
    if [ "$OPTIMIZE" -eq 1 ]; then
        echo "Stripping binary (Optimization enabled)..."
        strip "$OUT_PATH"
    fi
    
    echo "Compilation successful! Running ./$OUT_PATH..."
    echo "--------------------------------------------------"
    ./"$OUT_PATH"
    RET=$?
    echo "--------------------------------------------------"
    exit $RET
else
    echo "Compilation failed."
    exit 1
fi
