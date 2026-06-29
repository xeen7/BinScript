#!/bin/bash
./tests/test_raii_rethrow_bin_unopt > output.log 2>&1 &
PID=$!
sleep 0.5
top -b -n 1 -H -p $PID
kill -9 $PID
cat output.log
