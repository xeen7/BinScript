import sys

with open("crates/mir/src/lower/stmt.rs", "r") as f:
    content = f.read()

content = content.replace("use super::LowerCtx;", "use super::{LowerCtx, LoopStackFrame};\nuse crate::builtins::BuiltinFn;")

with open("crates/mir/src/lower/stmt.rs", "w") as f:
    f.write(content)
