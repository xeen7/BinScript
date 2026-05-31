import os

with open('rt-stubs/src/objects/mod.rs', 'r') as f:
    mod_rs = f.read()

mod_rs = mod_rs.replace('pub mod constructors;', 'pub mod builtins;')

with open('rt-stubs/src/objects/mod.rs', 'w') as f:
    f.write(mod_rs)

with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

lib_rs = lib_rs.replace('objects::constructors::', 'objects::builtins::')

with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

os.remove('rt-stubs/src/objects/constructors.rs')

print("Updated mod.rs and lib.rs, removed constructors.rs")
