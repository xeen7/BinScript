import sys
import re

files = ["crates/mir/src/lower/stmt.rs", "crates/mir/src/lower/expr.rs"]

for file in files:
    with open(file, "r") as f:
        content = f.read()

    # Fix visibility
    content = re.sub(r'^\s*fn lower_', '    pub(super) fn lower_', content, flags=re.MULTILINE)

    # Fix mismatched types for get(src_reg) -> get(&src_reg)
    content = content.replace("get(src_reg)", "get(&src_reg)")
    
    # Fix 'expected `&str`, found `String`' etc if any, but the compiler just pointed out `get(src_reg)`
    
    with open(file, "w") as f:
        f.write(content)
