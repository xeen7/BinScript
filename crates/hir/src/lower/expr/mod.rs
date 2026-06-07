use oxc::ast::ast::*;

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
    pub(crate) fn lower_expr(&mut self, expr: &Expression) -> CompileResult<HirExpr> {
        match expr {
            Expression::BooleanLiteral(_) |
            Expression::NullLiteral(_) |
            Expression::NumericLiteral(_) |
            Expression::BigIntLiteral(_) |
            Expression::RegExpLiteral(_) |
            Expression::StringLiteral(_) => self.lower_expr_lit(expr),
            Expression::ObjectExpression(obj) => self.lower_expr_object(obj),
            Expression::ThisExpression(t) => self.lower_expr_this(t),
            Expression::Identifier(id) => self.lower_expr_ident(id),
            Expression::BinaryExpression(bin) => self.lower_expr_bin(bin),
            Expression::LogicalExpression(logical) => self.lower_expr_logical(logical),
            Expression::UnaryExpression(u) => self.lower_expr_unary(u),
            Expression::CallExpression(call) => self.lower_expr_call(call),
            Expression::NewExpression(n) => self.lower_expr_new(n),
            m if m.is_member_expression() => self.lower_expr_member(m.as_member_expression().unwrap()),
            Expression::ParenthesizedExpression(p) => self.lower_expr_paren(p),
            Expression::AssignmentExpression(a) => self.lower_expr_assign(a),
            Expression::UpdateExpression(u) => self.lower_expr_update(u),
            Expression::ConditionalExpression(c) => self.lower_expr_cond(c),
            Expression::TemplateLiteral(tpl) => self.lower_expr_tpl(tpl),
            Expression::YieldExpression(y) => self.lower_expr_yield(y),
            Expression::SequenceExpression(seq) => self.lower_expr_seq(seq),
            Expression::ArrowFunctionExpression(arrow) => self.lower_expr_arrow(arrow),
            Expression::FunctionExpression(fn_expr) => self.lower_expr_fn(fn_expr),
            Expression::AwaitExpression(a) => self.lower_expr_await(a),
            Expression::ArrayExpression(arr) => self.lower_expr_array(arr),
            Expression::ChainExpression(o) => self.lower_expr_opt_chain(o),
            Expression::MetaProperty(mp) => self.lower_expr_meta_prop(mp),
            Expression::TaggedTemplateExpression(tt) => self.lower_expr_tagged_tpl(tt),
            Expression::ClassExpression(ce) => self.lower_expr_class(ce),
            Expression::Super(_) => self.lower_expr_super_prop(expr),
            // TS type assertions just lower the inner expression
            Expression::TSAsExpression(e) => self.lower_expr(&e.expression),
            Expression::TSSatisfiesExpression(e) => self.lower_expr(&e.expression),
            Expression::TSNonNullExpression(e) => self.lower_expr(&e.expression),
            Expression::TSTypeAssertion(e) => self.lower_expr(&e.expression),
            Expression::TSInstantiationExpression(e) => self.lower_expr(&e.expression),
            
            e => Err(CompileError::Lowering {
                message: format!("Unsupported expression in Stage 3: {:?}", e),
            }),
        }
    }
}
