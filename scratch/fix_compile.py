import os

with open('rt-stubs/src/objects/builtins/number.rs', 'r') as f:
    number_rs = f.read()
if 'TAG_MASK' not in number_rs[:200]:
    number_rs = number_rs.replace('use crate::gc;', 'use crate::gc;\nconst TAG_MASK: u64 = 0xFFFF_0000_0000_0000;\n')
with open('rt-stubs/src/objects/builtins/number.rs', 'w') as f:
    f.write(number_rs)

with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

lib_rs = lib_rs.replace('__bs_Error_new_n,', '')
lib_rs = lib_rs.replace('__bs_Error_new_2,', '')
with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

