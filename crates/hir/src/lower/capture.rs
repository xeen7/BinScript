//! Closure capture propagation: walk the HIR tree and fill in capture lists.

use std::collections::{HashMap, HashSet};
use crate::types::*;

pub(crate) fn populate_closure_captures(stmts: &mut [HirStmt], func_captures: &HashMap<FuncId, HashSet<BindingId>>) {
    for stmt in stmts {
        walk_stmt(stmt, func_captures);
    }
}

fn walk_stmt(stmt: &mut HirStmt, func_captures: &HashMap<FuncId, HashSet<BindingId>>) {
    match stmt {
        HirStmt::Expr(expr) => walk_expr(expr, func_captures),
        HirStmt::Let { init, .. } => {
            if let Some(expr) = init {
                walk_expr(expr, func_captures);
            }
        }
        HirStmt::Assign { value, .. } => walk_expr(value, func_captures),
        HirStmt::If { cond, then_body, else_body } => {
            walk_expr(cond, func_captures);
            for s in then_body {
                walk_stmt(s, func_captures);
            }
            if let Some(else_b) = else_body {
                for s in else_b {
                    walk_stmt(s, func_captures);
                }
            }
        }
        HirStmt::While { cond, body } => {
            walk_expr(cond, func_captures);
            for s in body {
                walk_stmt(s, func_captures);
            }
        }
        HirStmt::DoWhile { body, cond } => {
            for s in body {
                walk_stmt(s, func_captures);
            }
            walk_expr(cond, func_captures);
        }
        HirStmt::For { init, cond, update, body } => {
            if let Some(i) = init {
                walk_stmt(i, func_captures);
            }
            if let Some(c) = cond {
                walk_expr(c, func_captures);
            }
            if let Some(u) = update {
                walk_expr(u, func_captures);
            }
            for s in body {
                walk_stmt(s, func_captures);
            }
        }
        HirStmt::ForOf { left, right, body, is_await: _ } => {
            walk_stmt(left, func_captures);
            walk_expr(right, func_captures);
            for s in body {
                walk_stmt(s, func_captures);
            }
        }
        HirStmt::Return(opt_expr) => {
            if let Some(expr) = opt_expr {
                walk_expr(expr, func_captures);
            }
        }
        HirStmt::Break(_) | HirStmt::Continue(_) => {}
        HirStmt::Labeled { body, .. } => {
            walk_stmt(body, func_captures);
        }
        HirStmt::Block(body) => {
            for s in body {
                walk_stmt(s, func_captures);
            }
        }
        HirStmt::FuncDecl { body, .. } => {
            for s in body {
                walk_stmt(s, func_captures);
            }
        }
        HirStmt::Throw(expr) => {
            walk_expr(expr, func_captures);
        }
        HirStmt::Try { body, catch_body, finally_body, .. } => {
            for s in body {
                walk_stmt(s, func_captures);
            }
            for s in catch_body {
                walk_stmt(s, func_captures);
            }
            if let Some(fin) = finally_body {
                for s in fin {
                    walk_stmt(s, func_captures);
                }
            }
        }
        HirStmt::Switch { discriminant, cases } => {
            walk_expr(discriminant, func_captures);
            for case in cases {
                if let Some(test) = &mut case.test {
                    walk_expr(test, func_captures);
                }
                for s in &mut case.consequent {
                    walk_stmt(s, func_captures);
                }
            }
        }
    }
}

fn walk_expr(expr: &mut HirExpr, func_captures: &HashMap<FuncId, HashSet<BindingId>>) {
    match expr {
        HirExpr::Lit(_) | HirExpr::Var(_) | HirExpr::GlobalRef(_) | HirExpr::JsonTape(_) => {}
        HirExpr::BinOp(_, left, right) => {
            walk_expr(left, func_captures);
            walk_expr(right, func_captures);
        }
        HirExpr::UnaryOp(_, arg) => {
            walk_expr(arg, func_captures);
        }
        HirExpr::Call { callee, args } => {
            walk_expr(callee, func_captures);
            for arg in args {
                walk_expr(arg, func_captures);
            }
        }
        HirExpr::Assign { value, .. } => {
            walk_expr(value, func_captures);
        }
        HirExpr::Ternary { cond, then_expr, else_expr } => {
            walk_expr(cond, func_captures);
            walk_expr(then_expr, func_captures);
            walk_expr(else_expr, func_captures);
        }
        HirExpr::Seq(exprs) => {
            for e in exprs {
                walk_expr(e, func_captures);
            }
        }
        HirExpr::MemberCall { args, .. } => {
            for arg in args {
                walk_expr(arg, func_captures);
            }
        }
        HirExpr::MemberGet { object, .. } => {
            walk_expr(object, func_captures);
        }
        HirExpr::MemberSet { object, value, .. } => {
            walk_expr(object, func_captures);
            walk_expr(value, func_captures);
        }
        HirExpr::CompoundMemberSet { object, value, .. } => {
            walk_expr(object, func_captures);
            walk_expr(value, func_captures);
        }
        HirExpr::New { args, .. } => {
            for arg in args {
                walk_expr(arg, func_captures);
            }
        }
        HirExpr::InstanceOf { expr, .. } => {
            walk_expr(expr, func_captures);
        }
        HirExpr::MethodCall { object, args, .. } => {
            walk_expr(object, func_captures);
            for arg in args {
                walk_expr(arg, func_captures);
            }
        }
        HirExpr::Closure { func_id, captures } => {
            if let Some(c_set) = func_captures.get(func_id) {
                let mut caps_vec: Vec<BindingId> = c_set.iter().cloned().collect();
                caps_vec.sort();
                *captures = caps_vec;
            }
        }
        HirExpr::Yield { arg, .. } => {
            if let Some(expr) = arg {
                walk_expr(expr, func_captures);
            }
        }
        HirExpr::Await(expr) => {
            walk_expr(expr, func_captures);
        }
        HirExpr::ArrayLit(elems) => {
            for e in elems {
                walk_expr(e, func_captures);
            }
        }
        HirExpr::IndexGet { object, index } => {
            walk_expr(object, func_captures);
            walk_expr(index, func_captures);
        }
        HirExpr::IndexSet { object, index, value } => {
            walk_expr(object, func_captures);
            walk_expr(index, func_captures);
            walk_expr(value, func_captures);
        }
        HirExpr::CompoundIndexSet { object, index, value, .. } => {
            walk_expr(object, func_captures);
            walk_expr(index, func_captures);
            walk_expr(value, func_captures);
        }
        HirExpr::Spread(inner) => {
            walk_expr(inner, func_captures);
        }
        HirExpr::DeleteProp { object, property } => {
            walk_expr(object, func_captures);
            walk_expr(property, func_captures);
        }
    }
}
