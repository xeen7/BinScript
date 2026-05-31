import sys

with open("crates/mir/src/lower/expr.rs", "r") as f:
    content = f.read()

content = """use diagnostics::{CompileError, CompileResult};
use hir::{HirExpr, Literal, UnaryOp as HUnaryOp, BinOp as HBinOp};
use crate::builtins::BuiltinFn;
use crate::types::*;
use super::LowerCtx;

impl<'a> LowerCtx<'a> {
""" + content + "}\n"

with open("crates/mir/src/lower/expr.rs", "w") as f:
    f.write(content)
