import os

with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

lib_rs = lib_rs.replace('__bs_RegExp_new, __bs_Error_new_0, __bs_Error_new_1,', '__bs_RegExp_new,')
with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

with open('rt-stubs/src/types/string_utils.rs', 'r') as f:
    string_utils = f.read()

string_utils = string_utils.replace('crate::exception::__bs_URIError_new', 'crate::objects::builtins::__bs_URIError_new')

with open('rt-stubs/src/types/string_utils.rs', 'w') as f:
    f.write(string_utils)
