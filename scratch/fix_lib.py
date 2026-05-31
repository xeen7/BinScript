import os

with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

lib_rs = lib_rs.replace('pub mod collections;\n', '')
lib_rs = lib_rs.replace('pub mod math;\n', '')
lib_rs = lib_rs.replace('pub mod exception;\n', 'pub mod exception;\n') # leave exception

# Actually math and collections aren't imported if we deleted the mod statements. Let's see what is left to fix.
# There might be explicit `pub use` for map/set/math that we need to add to `pub use objects::builtins::{ ... }`

# the builtins `{ ... }` block:
builtins_block_start = lib_rs.find('pub use objects::builtins::{')
if builtins_block_start != -1:
    builtins_block_end = lib_rs.find('};', builtins_block_start)
    old_builtins = lib_rs[builtins_block_start:builtins_block_end+2]
    
    new_builtins = old_builtins.replace('};', '    __bs_Map_new_0, __bs_Map_new_1, __bs_Set_new_0, __bs_Set_new_1,\n    __bs_WeakMap_new_0, __bs_WeakMap_new_1, __bs_WeakSet_new_0, __bs_WeakSet_new_1,\n    __bs_RegExp_new, __bs_Error_new_0, __bs_Error_new_1, __bs_Error_new_2, __bs_Error_new_n,\n    __bs_math_floor, __bs_math_ceil, __bs_math_round, __bs_math_abs, __bs_math_sqrt,\n    __bs_math_pow, __bs_math_min, __bs_math_max, __bs_math_log, __bs_math_log2,\n    __bs_math_sin, __bs_math_cos, __bs_math_tan, __bs_math_random, __bs_math_trunc,\n    __bs_parseInt, __bs_parseInt_1, __bs_parseInt_2, __bs_parseFloat, __bs_isNaN, __bs_isFinite,\n    __bs_number_isInteger, __bs_number_isSafeInteger,\n};')
    
    lib_rs = lib_rs.replace(old_builtins, new_builtins)

with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

print("Updated lib.rs")
