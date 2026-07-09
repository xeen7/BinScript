#!/bin/bash
export RUST_BACKTRACE=1
./run_test.sh tests/examples/t19_even_more_daily_use_cases.ts > /dev/null 2>&1
gdb -q -ex run -ex bt -ex quit ./build/t19_even_more_daily_use_cases
