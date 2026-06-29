#!/bin/bash
./tests/test_raii_rethrow_bin_unopt > output_gdb.log 2>&1 &
PID=$!
sleep 0.5
gdb -batch -ex "bt" -p $PID
kill -9 $PID
cat output_gdb.log
