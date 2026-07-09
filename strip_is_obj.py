import re

with open("rt-stubs/src/objects/builtins/object.rs", "r") as f:
    code = f.read()

# Replace tag checks
def remove_is_object(code, search_tag, ptr_name):
    # This removes the `is_target_object = if ...` and the `if !is_target_object` lines.
    pattern = r"let (is_[a-zA-Z0-9_]+) = if ([a-zA-Z0-9_]+) == 0xFFFC_0000_0000_0000 \{\s+let header = .*?\s+let header = header\.wrapping_sub.*?\s+let flags = unsafe \{ \(\*header\)\.flags\.load.*?\};\s+\(flags & crate::circ::VTABLE_PTR\) != 0\s+\} else \{\s+true\s+\};\s+if !\1 \{[^\}]*\}\n"
    code = re.sub(pattern, "", code)
    
    # Also remove `is_object` for vtable checks inside `__bs_object_keys` etc.
    pattern2 = r"let (is_[a-zA-Z0-9_]+) = if ([a-zA-Z0-9_]+) == 0xFFFC_0000_0000_0000 \{\s+let header = .*?\.wrapping_sub.*?\s+let flags = unsafe \{ \(\*header\)\.flags\.load.*?\};\s+\(flags & crate::circ::VTABLE_PTR\) != 0\s+\} else \{\s+true\s+\};\s+if \1 \{"
    code = re.sub(pattern2, "", code)
    
    return code

code = remove_is_object(code, "", "")

# Now I need to fix the unbalanced brackets where `if is_object {` was removed.
# This might be tricky. Let me just replace the exact blocks.
