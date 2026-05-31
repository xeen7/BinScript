import sys

with open("crates/mir/src/lower/stmt.rs", "r") as f:
    content = f.read()

content = """use diagnostics::{CompileError, CompileResult};
use hir::{HirStmt, HirExpr};
use crate::types::*;
use super::LowerCtx;

impl<'a> LowerCtx<'a> {
""" + content + "}\n"

with open("crates/mir/src/lower/stmt.rs", "w") as f:
    f.write(content)
