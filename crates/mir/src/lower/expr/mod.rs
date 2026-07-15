use diagnostics::CompileResult;
use hir::{self, HirExpr};
use crate::types::*;
use super::LowerCtx;

mod ops;
mod access;
mod call;
mod control;
mod literal;
mod var;

impl<'a> LowerCtx<'a> {
    pub(crate) fn lower_expr(&mut self, expr: &HirExpr) -> CompileResult<MirOperand> {
        match expr {
            // Primitive / Literals
            HirExpr::Lit(lit)                  => self.lower_expr_lit(lit),
            HirExpr::ArrayLit(elements)        => self.lower_expr_array_lit(elements),
            HirExpr::JsonTape(tape)            => self.lower_expr_json_tape(tape),
            HirExpr::Spread(_)                 => self.lower_expr_spread(),

            // Variables / References
            HirExpr::Var(id)                   => self.lower_expr_var(id),
            HirExpr::GlobalRef(name)           => self.lower_expr_global_ref(name),
            HirExpr::Assign { target, value }  => self.lower_expr_assign(target, value),

            // Object/Property Access
            HirExpr::MemberGet { object, property }        => self.lower_expr_member_get(object, property),
            HirExpr::MemberSet { object, property, value } => self.lower_expr_member_set(object, property, value),
            HirExpr::CompoundMemberSet { object, property, op, value } => self.lower_expr_compound_member_set(object, property, op, value),
            HirExpr::IndexGet { object, index }            => self.lower_expr_index_get(object, index),
            HirExpr::IndexSet { object, index, value }     => self.lower_expr_index_set(object, index, value),
            HirExpr::CompoundIndexSet { object, index, op, value } => self.lower_expr_compound_index_set(object, index, op, value),
            HirExpr::DeleteProp { object, property }       => self.lower_expr_delete_prop(object, property),

            // Invocations
            HirExpr::Call { callee, args }                 => self.lower_expr_call(callee, args),
            HirExpr::MemberCall { object, method, args }   => self.lower_expr_member_call(object, method, args),
            HirExpr::MethodCall { object, method, args }   => self.lower_expr_method_call(object, method, args),
            HirExpr::New { class_name, args }              => self.lower_expr_new(class_name, args),
            HirExpr::Closure { func_id, captures }         => self.lower_expr_closure(func_id, captures),

            // Operators
            HirExpr::BinOp(op, left, right) => self.lower_expr_bin_op(op, left, right),
            HirExpr::UnaryOp(op, arg)       => self.lower_expr_unary_op(op, arg),
            HirExpr::Ternary { cond, then_expr, else_expr } => self.lower_expr_ternary(cond, then_expr, else_expr),
            HirExpr::InstanceOf { expr, class_name } => self.lower_expr_instance_of(expr, class_name),

            // Control flow
            HirExpr::Seq(exprs)                => self.lower_expr_seq(exprs),
            HirExpr::Yield { arg, delegate }   => self.lower_expr_yield(arg, *delegate),
            HirExpr::Await(arg)                => self.lower_expr_await(arg),
        }
    }
}
