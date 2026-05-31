import os
import shutil

os.makedirs('rt-stubs/src/array', exist_ok=True)
os.makedirs('rt-stubs/src/string', exist_ok=True)
os.makedirs('rt-stubs/src/json', exist_ok=True)
os.makedirs('rt-stubs/src/promise', exist_ok=True)

# 1. Array
shutil.move('rt-stubs/src/array.rs', 'rt-stubs/src/array/mod.rs')

# 2. String
shutil.move('rt-stubs/src/string_methods.rs', 'rt-stubs/src/string/mod.rs')

# 3. JSON
shutil.move('rt-stubs/src/json.rs', 'rt-stubs/src/json/mod.rs')
shutil.move('rt-stubs/src/json_tape.rs', 'rt-stubs/src/json/tape.rs')

# 4. Promise
shutil.move('rt-stubs/src/promise.rs', 'rt-stubs/src/promise/mod.rs')
shutil.move('rt-stubs/src/promise_combinators.rs', 'rt-stubs/src/promise/combinators.rs')
shutil.move('rt-stubs/src/microtask.rs', 'rt-stubs/src/promise/microtask.rs')

# Update lib.rs
with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

# Replace top-level modules
lib_rs = lib_rs.replace('pub mod string_methods;', 'pub mod string;')
lib_rs = lib_rs.replace('pub mod json_tape;\n', '')
lib_rs = lib_rs.replace('pub mod promise_combinators;\n', '')
lib_rs = lib_rs.replace('pub mod microtask;\n', '')

with open('rt-stubs/src/lib.rs', 'w') as f:
    f.write(lib_rs)

# Update imports in all files
def replace_in_files(replacements):
    for root, _, files in os.walk('rt-stubs/src'):
        for file in files:
            if not file.endswith('.rs'): continue
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()
            original = content
            for old, new in replacements:
                content = content.replace(old, new)
            if content != original:
                with open(path, 'w') as f:
                    f.write(content)

replacements = [
    ('crate::json_tape', 'crate::json::tape'),
    ('crate::promise_combinators', 'crate::promise::combinators'),
    ('crate::microtask', 'crate::promise::microtask'),
    ('crate::string_methods', 'crate::string'),
]

replace_in_files(replacements)

# Add pub mod tape to json/mod.rs
with open('rt-stubs/src/json/mod.rs', 'r') as f:
    json_mod = f.read()
with open('rt-stubs/src/json/mod.rs', 'w') as f:
    f.write('pub mod tape;\n' + json_mod)

# Add pub mod combinators and pub mod microtask to promise/mod.rs
with open('rt-stubs/src/promise/mod.rs', 'r') as f:
    promise_mod = f.read()
with open('rt-stubs/src/promise/mod.rs', 'w') as f:
    f.write('pub mod combinators;\npub mod microtask;\n' + promise_mod)

print("Done moving files and updating imports.")
