use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_switch(&mut self, s: &SwitchStatement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let discriminant = self.lower_expr(&s.discriminant)?;
        
        let mut cases = Vec::new();
        for case in &s.cases {
            let test = match &case.test {
                Some(expr) => Some(self.lower_expr(expr)?),
                None => None,
            };
            
            let mut consequent = Vec::new();
            for stmt in &case.consequent {
                self.lower_stmt(stmt, &mut consequent)?;
            }
            
            cases.push(HirSwitchCase { test, consequent });
        }
        
        out.push(HirStmt::Switch { discriminant, cases });
        Ok(())
    }
}
