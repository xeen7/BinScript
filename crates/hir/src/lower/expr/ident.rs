use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_ident(&mut self, id: &Ident) -> CompileResult<HirExpr> {
        let name = id.sym.to_string();
        if let Some(aliased) = self.function_aliases.get(&name) {
            Ok(HirExpr::GlobalRef(aliased.clone()))
        } else {
            match self.lookup(&name) {
                Some(bid) => {
                    // If this identifier refers to the class currently being lowered,
                    // emit a GlobalRef to avoid capturing the class binding as a closure
                    // variable (which would cause a segfault since class methods are
                    // called without an __env pointer).
                    if let Some((ref class_name, class_bid)) = self.current_class {
                        if bid == class_bid {
                            return Ok(HirExpr::GlobalRef(format!("__bs_class_val_{}", class_name)));
                        }
                    }
                    self.record_lookup(bid);
                    Ok(HirExpr::Var(bid))
                }
                None => Ok(HirExpr::GlobalRef(name)),
            }
        }
    }
}
