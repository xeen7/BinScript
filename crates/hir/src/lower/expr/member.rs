use swc_core::ecma::ast::*;

use diagnostics::{CompileError, CompileResult};
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_expr_member(&mut self, m: &MemberExpr) -> CompileResult<HirExpr> {
        if let Expr::Ident(id) = &*m.obj {
            let obj_name = id.sym.to_string();
            if obj_name == "Math" {
                if let MemberProp::Ident(prop_id) = &m.prop {
                    let prop_name = prop_id.sym.to_string();
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
                if let MemberProp::Ident(prop_id) = &m.prop {
                    let prop_name = prop_id.sym.to_string();
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

        let obj = self.lower_expr(&m.obj)?;
        match &m.prop {
            MemberProp::Ident(prop_id) => {
                Ok(HirExpr::MemberGet {
                    object: Box::new(obj),
                    property: prop_id.sym.to_string(),
                })
            }
            MemberProp::Computed(computed) => {
                let idx = self.lower_expr(&computed.expr)?;
                Ok(HirExpr::IndexGet {
                    object: Box::new(obj),
                    index: Box::new(idx),
                })
            }
            MemberProp::PrivateName(pn) => {
                Ok(HirExpr::MemberGet {
                    object: Box::new(obj),
                    property: format!("__private_{}", pn.name),
                })
            }
        }
    }

    pub(super) fn lower_expr_super_prop(&mut self, sp: &SuperPropExpr) -> CompileResult<HirExpr> {
        if let Some(this_id) = self.this_binding {
            match &sp.prop {
                SuperProp::Ident(prop_id) => {
                    Ok(HirExpr::MemberGet {
                        object: Box::new(HirExpr::Var(this_id)),
                        property: prop_id.sym.to_string(),
                    })
                }
                SuperProp::Computed(computed) => {
                    let idx = self.lower_expr(&computed.expr)?;
                    Ok(HirExpr::IndexGet {
                        object: Box::new(HirExpr::Var(this_id)),
                        index: Box::new(idx),
                    })
                }
            }
        } else {
            Err(CompileError::Lowering {
                message: "super property access outside class method/constructor".into(),
            })
        }
    }
}
