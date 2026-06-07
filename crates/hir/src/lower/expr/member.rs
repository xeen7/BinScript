use oxc::ast::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_member(&mut self, m: &MemberExpression) -> CompileResult<HirExpr> {
        if let Expression::Identifier(id) = m.object() {
            let obj_name = id.name.to_string();
            if obj_name == "Math" {
                if let MemberExpression::StaticMemberExpression(prop) = m {
                    let prop_name = prop.property.name.to_string();
                    let val = match prop_name.as_str() {
                        "PI" => Some(std::f64::consts::PI),
                        "E" => Some(std::f64::consts::E),
                        "LN10" => Some(std::f64::consts::LN_10),
                        "LN2" => Some(std::f64::consts::LN_2),
                        "LOG10E" => Some(std::f64::consts::LOG10_E),
                        "LOG2E" => Some(std::f64::consts::LOG2_E),
                        "SQRT1_2" => Some(std::f64::consts::FRAC_1_SQRT_2),
                        "SQRT2" => Some(std::f64::consts::SQRT_2),
                        _ => None,
                    };
                    if let Some(num) = val {
                        return Ok(HirExpr::Lit(Literal::Number(num)));
                    }
                }
            } else if obj_name == "Number" {
                if let MemberExpression::StaticMemberExpression(prop) = m {
                    let prop_name = prop.property.name.to_string();
                    let val = match prop_name.as_str() {
                        "MAX_VALUE" => Some(std::f64::MAX),
                        "MIN_VALUE" => Some(std::f64::MIN_POSITIVE),
                        "NaN" => Some(std::f64::NAN),
                        "POSITIVE_INFINITY" => Some(std::f64::INFINITY),
                        "NEGATIVE_INFINITY" => Some(std::f64::NEG_INFINITY),
                        "EPSILON" => Some(std::f64::EPSILON),
                        _ => None,
                    };
                    if let Some(num) = val {
                        return Ok(HirExpr::Lit(Literal::Number(num)));
                    }
                }
            }
        }

        let obj = self.lower_expr(m.object())?;
        match m {
            MemberExpression::StaticMemberExpression(prop) => {
                Ok(HirExpr::MemberGet {
                    object: Box::new(obj),
                    property: prop.property.name.to_string(),
                })
            }
            MemberExpression::ComputedMemberExpression(computed) => {
                let idx = self.lower_expr(&computed.expression)?;
                Ok(HirExpr::IndexGet {
                    object: Box::new(obj),
                    index: Box::new(idx),
                })
            }
            MemberExpression::PrivateFieldExpression(pn) => {
                Ok(HirExpr::MemberGet {
                    object: Box::new(obj),
                    property: format!("__private_{}", pn.field.name),
                })
            }
        }
    }

    pub(super) fn lower_expr_super_prop(&mut self, _sp: &Expression) -> CompileResult<HirExpr> {
        if let Some(this_id) = self.this_binding {
            Ok(HirExpr::Var(this_id))
        } else {
            Err(CompileError::Lowering {
                message: "super used outside class method/constructor".into(),
            })
        }
    }
}
