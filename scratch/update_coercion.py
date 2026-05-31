import os

with open('rt-stubs/src/types/coercion.rs', 'r') as f:
    coercion_rs = f.read()

coercion_rs = coercion_rs.replace('crate::objects::constructors::', 'crate::objects::builtins::')

with open('rt-stubs/src/types/coercion.rs', 'w') as f:
    f.write(coercion_rs)

with open('rt-stubs/src/types/symbol.rs', 'r') as f:
    symbol_rs = f.read()

symbol_rs = symbol_rs.replace('crate::objects::constructors::', 'crate::objects::builtins::')

with open('rt-stubs/src/types/symbol.rs', 'w') as f:
    f.write(symbol_rs)

print("Updated coercion.rs and symbol.rs")
