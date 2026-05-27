use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

mod lit;
mod object;
mod this;
mod ident;
mod bin;
mod unary;
mod call;
mod new_expr;
mod member;
mod paren;
mod assign;
mod update;
mod cond;
mod tpl;
mod yield_expr;
mod seq;
mod arrow;
mod fn_expr;
mod await_expr;
mod array;
mod opt_chain;
mod meta_prop;
mod class;


impl LowerCtx {
    pub(crate) fn lower_expr(&mut self, expr: &Expr) -> CompileResult<HirExpr> {
        match expr {
            Expr::Lit(lit) => self.lower_expr_lit(lit),
            Expr::Object(obj) => self.lower_expr_object(obj),
            Expr::This(t) => self.lower_expr_this(t),
            Expr::Ident(id) => self.lower_expr_ident(id),
            Expr::Bin(bin) => self.lower_expr_bin(bin),
            Expr::Unary(u) => self.lower_expr_unary(u),
            Expr::Call(call) => self.lower_expr_call(call),
            Expr::New(n) => self.lower_expr_new(n),
            Expr::Member(m) => self.lower_expr_member(m),
            Expr::Paren(p) => self.lower_expr_paren(p),
            Expr::Assign(a) => self.lower_expr_assign(a),
            Expr::Update(u) => self.lower_expr_update(u),
            Expr::Cond(c) => self.lower_expr_cond(c),
            Expr::Tpl(tpl) => self.lower_expr_tpl(tpl),
            Expr::Yield(y) => self.lower_expr_yield(y),
            Expr::Seq(seq) => self.lower_expr_seq(seq),
            Expr::Arrow(arrow) => self.lower_expr_arrow(arrow),
            Expr::Fn(fn_expr) => self.lower_expr_fn(fn_expr),
            Expr::Await(a) => self.lower_expr_await(a),
            Expr::Array(arr) => self.lower_expr_array(arr),
            Expr::SuperProp(sp) => self.lower_expr_super_prop(sp),
            Expr::OptChain(o) => self.lower_expr_opt_chain(o),
            Expr::MetaProp(mp) => self.lower_expr_meta_prop(mp),
            Expr::TaggedTpl(tt) => self.lower_expr_tagged_tpl(tt),
            Expr::Class(ce) => self.lower_expr_class(ce),
            _ => Err(CompileError::Lowering {
                message: format!("Unsupported expression in Stage 3"),
            }),
        }
    }
}
