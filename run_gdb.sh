#!/bin/bash
gdb -batch -ex 'b ts_rt_stubs::slab::fast_free_shared' -ex 'commands' -ex 'p alloc_size' -ex 'p ptr' -ex 'bt 5' -ex 'c' -ex 'end' -ex 'run' ./build/t12_minimal_async
