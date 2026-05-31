import os
import shutil

os.makedirs('rt-stubs/src/math', exist_ok=True)
os.makedirs('rt-stubs/src/collections', exist_ok=True)
os.makedirs('rt-stubs/src/exception', exist_ok=True)

# 1. Math
shutil.move('rt-stubs/src/math_global.rs', 'rt-stubs/src/math/mod.rs')

# 2. Collections
shutil.move('rt-stubs/src/collections.rs', 'rt-stubs/src/collections/mod.rs')

# 3. Exception
shutil.move('rt-stubs/src/exception.rs', 'rt-stubs/src/exception/mod.rs')

# Update lib.rs
with open('rt-stubs/src/lib.rs', 'r') as f:
    lib_rs = f.read()

lib_rs = lib_rs.replace('pub mod math_global;', 'pub mod math;')
lib_rs = lib_rs.replace('use math_global', 'use math')

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
    ('crate::math_global', 'crate::math'),
    ('crate::collections', 'crate::collections'), # this one is fine
    ('crate::exception', 'crate::exception'),     # this one is fine
]
replace_in_files(replacements)

print("Done moving remaining files.")
