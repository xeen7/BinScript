#!/bin/bash
sed -i 's/console.log("keys length: " + Object.keys(errors).length);/const debug_keys = Object.keys(errors);\n  console.log("keys length: " + debug_keys.length);\n  console.log("keys[0] = " + debug_keys[0]);/' tests/examples/t18_more_daily_use_cases.ts
./run_test.sh tests/examples/t18_more_daily_use_cases.ts
