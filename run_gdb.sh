#!/bin/bash
cargo build -p compiler-core
./build/t5 &
PID=$!
sleep 1
gdb -batch -ex "bt" -p $PID
kill -9 $PID
