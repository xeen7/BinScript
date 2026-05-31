//! Module-level lowering: `lower_module` and `lower_module_decl`.

use swc_core::ecma::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use super::context::LowerCtx;
use super::capture::populate_closure_captures;

impl LowerCtx {
    pub(crate) fn lower_module(&mut self, module: &Module) -> CompileResult<HirModule> {
        // Pre-collect all function, method, and constructor names in the module
        for item in &module.body {
            let decl_opt = match item {
                ModuleItem::Stmt(Stmt::Decl(decl)) => Some(decl),
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => Some(&export.decl),
                _ => None,
            };
            if let Some(decl) = decl_opt {
                match decl {
                    Decl::Fn(fn_decl) => {
                        self.function_names.insert(fn_decl.ident.sym.to_string());
                    }
                    Decl::Class(class_decl) => {
                        let class_name = class_decl.ident.sym.to_string();
                        self.function_names.insert(format!("__bs_class_{}_constructor", class_name));
                        for member in &class_decl.class.body {
                            if let ClassMember::Method(method) = member {
                                if !method.is_static {
                                    let m_name = match &method.key {
                                        PropName::Ident(id) => id.sym.to_string(),
                                        PropName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
                                        PropName::Num(n) => n.value.to_string(),
                                        _ => "unknown".to_string(),
                                    };
                                    self.function_names.insert(format!("__bs_class_{}_{}", class_name, m_name));
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            let default_decl_opt = match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDefaultDecl(export)) => Some(&export.decl),
                _ => None,
            };
            if let Some(default_decl) = default_decl_opt {
                match default_decl {
                    DefaultDecl::Fn(fn_expr) => {
                        if let Some(ident) = &fn_expr.ident {
                            self.function_names.insert(ident.sym.to_string());
                        }
                    }
                    DefaultDecl::Class(class_expr) => {
                        if let Some(ident) = &class_expr.ident {
                            let class_name = ident.sym.to_string();
                            self.function_names.insert(format!("__bs_class_{}_constructor", class_name));
                            for member in &class_expr.class.body {
                                if let ClassMember::Method(method) = member {
                                    if !method.is_static {
                                        let m_name = match &method.key {
                                            PropName::Ident(id) => id.sym.to_string(),
                                            PropName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
                                            PropName::Num(n) => n.value.to_string(),
                                            _ => "unknown".to_string(),
                                        };
                                        self.function_names.insert(format!("__bs_class_{}_{}", class_name, m_name));
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut stmts = Vec::new();
        for item in &module.body {
            match item {
                ModuleItem::Stmt(stmt) => {
                    self.lower_stmt(stmt, &mut stmts)?;
                }
                ModuleItem::ModuleDecl(decl) => {
                    self.lower_module_decl(decl, &mut stmts)?;
                }
            }
        }

        let mut capture_cells = std::collections::HashSet::new();
        for captures in self.func_captures.values() {
            for &bid in captures {
                if self.reassigned_bindings.contains(&bid) {
                    capture_cells.insert(bid);
                }
            }
        }

        for f in &mut self.functions {
            let mut caps = Vec::new();
            if let Some(c_set) = self.func_captures.get(&f.id) {
                let mut caps_vec: Vec<BindingId> = c_set.iter().cloned().collect();
                caps_vec.sort();
                caps = caps_vec;
            }
            f.captures = caps;
        }

        populate_closure_captures(&mut stmts, &self.func_captures);
        for f in &mut self.functions {
            populate_closure_captures(&mut f.body, &self.func_captures);
        }

        Ok(HirModule {
            stmts,
            functions: std::mem::take(&mut self.functions),
            classes: std::mem::take(&mut self.classes),
            capture_cells,
            next_binding_id: self.next_binding,
            next_func_id: self.next_func,
            exports: std::mem::take(&mut self.exports),
        })
    }

    pub(crate) fn lower_module_decl(&mut self, decl: &ModuleDecl, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match decl {
            ModuleDecl::Import(_) => {
                // Pre-resolved and injected by caller, so nothing to do here
                Ok(())
            }
            ModuleDecl::ExportDecl(export) => {
                // Lower the declaration normally
                self.lower_decl(&export.decl, out)?;
                
                // Track what was exported
                match &export.decl {
                    Decl::Var(var_decl) => {
                        for decl in &var_decl.decls {
                            if let Pat::Ident(binding_ident) = &decl.name {
                                let name = binding_ident.id.sym.to_string();
                                if let Some(bid) = self.lookup(&name) {
                                    self.exports.named.insert(name, bid);
                                }
                            }
                        }
                    }
                    Decl::Fn(fn_decl) => {
                        let name = fn_decl.ident.sym.to_string();
                        if let Some(bid) = self.lookup(&name) {
                            self.exports.named.insert(name.clone(), bid);
                        }
                        // Find the func_id
                        if let Some(f) = self.functions.iter().find(|f| f.name == name) {
                            self.exports.functions.insert(name, f.id);
                        }
                    }
                    Decl::Class(class_decl) => {
                        let name = class_decl.ident.sym.to_string();
                        if let Some(bid) = self.lookup(&name) {
                            self.exports.named.insert(name.clone(), bid);
                        }
                        self.exports.classes.insert(name.clone(), name);
                    }
                    _ => {}
                }
                Ok(())
            }
            ModuleDecl::ExportDefaultDecl(export) => {
                match &export.decl {
                    DefaultDecl::Fn(fn_expr) => {
                        let name = fn_expr.ident.as_ref()
                            .map(|id| id.sym.to_string())
                            .unwrap_or_else(|| "__bs_default_fn".to_string());
                        
                        // We lower the default fn like a standard named Fn expression or decl
                        let func_id = self.fresh_func_id();
                        let binding = self.declare(&name);
                        
                        self.function_stack.push(func_id);
                        self.push_scope();
                        let mut params = Vec::new();
                        for p in &fn_expr.function.params {
                            if let Pat::Ident(ident) = &p.pat {
                                let pname = ident.sym.to_string();
                                let pid = self.declare(&pname);
                                params.push((pid, pname));
                            }
                        }
                        let body = match &fn_expr.function.body {
                            Some(b) => self.lower_block_stmts(b)?,
                            None => Vec::new(),
                        };
                        self.pop_scope();
                        self.function_stack.pop();
                        
                        self.functions.push(HirFunction {
                            id: func_id,
                            name: name.clone(),
                            params,
                            body,
                            captures: Vec::new(),
                            is_generator: fn_expr.function.is_generator,
                            is_async: fn_expr.function.is_async,
                        });
                        
                        let mut hir_params = Vec::new();
                        for p in &fn_expr.function.params {
                            if let Pat::Ident(ident) = &p.pat {
                                let pname = ident.sym.to_string();
                                if let Some(pid) = self.lookup(&pname) {
                                    hir_params.push((pid, pname));
                                }
                            }
                        }

                        let body_copy = match &fn_expr.function.body {
                            Some(b) => self.lower_block_stmts(b).unwrap_or_default(),
                            None => Vec::new(),
                        };

                        out.push(HirStmt::FuncDecl {
                            id: func_id,
                            name: name.clone(),
                            params: hir_params,
                            body: body_copy,
                        });
                        
                        self.exports.default = Some(binding);
                        self.exports.functions.insert("default".to_string(), func_id);
                    }
                    DefaultDecl::Class(class_expr) => {
                        let name = class_expr.ident.as_ref()
                            .map(|id| id.sym.to_string())
                            .unwrap_or_else(|| "__bs_default_class".to_string());
                        
                        // Class is lowered using lower_class_decl
                        let binding = self.declare(&name);
                        let super_name = class_expr.class.super_class.as_ref().and_then(|expr| {
                            if let Expr::Ident(id) = &**expr {
                                Some(id.sym.to_string())
                            } else {
                                None
                            }
                        });
                        
                        let old_super = self.current_super.clone();
                        self.current_super = super_name.clone();
                        
                        let mut fields = Vec::new();
                        let mut methods = Vec::new();
                        let mut getters = Vec::new();
                        let mut setters = Vec::new();
                        
                        for member in &class_expr.class.body {
                            match member {
                                ClassMember::ClassProp(prop) => {
                                    if let PropName::Ident(id) = &prop.key {
                                        fields.push(id.sym.to_string());
                                    }
                                }
                                ClassMember::Method(method) => {
                                    if !method.is_static {
                                        let base_name = match &method.key {
                                            PropName::Ident(id) => id.sym.to_string(),
                                            PropName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
                                            PropName::Num(n) => n.value.to_string(),
                                            _ => "unknown".to_string(),
                                        };
                                        let m_name = match method.kind {
                                            MethodKind::Getter => {
                                                getters.push(base_name.clone());
                                                format!("__get_{}", base_name)
                                            }
                                            MethodKind::Setter => {
                                                setters.push(base_name.clone());
                                                format!("__set_{}", base_name)
                                            }
                                            MethodKind::Method => base_name,
                                        };
                                        let f_id = self.fresh_func_id();
                                        
                                        self.function_stack.push(f_id);
                                        self.push_scope();
                                        
                                        let old_this = self.this_binding;
                                        let this_id = self.declare("this");
                                        self.this_binding = Some(this_id);
                                        
                                        let mut params = Vec::new();
                                        params.push((this_id, "this".to_string()));
                                        
                                        for p in &method.function.params {
                                            if let Pat::Ident(ident) = &p.pat {
                                                let pname = ident.sym.to_string();
                                                let pid = self.declare(&pname);
                                                params.push((pid, pname));
                                            }
                                        }
                                        
                                        let f_body = match &method.function.body {
                                            Some(b) => self.lower_block_stmts(b)?,
                                            None => Vec::new(),
                                        };
                                        
                                        self.this_binding = old_this;
                                        self.pop_scope();
                                        self.function_stack.pop();
                                        
                                        let method_fn_name = format!("__bs_class_{}_{}", name, m_name);
                                        self.functions.push(HirFunction {
                                            id: f_id,
                                            name: method_fn_name.clone(),
                                            params,
                                            body: f_body,
                                            captures: Vec::new(),
                                            is_generator: method.function.is_generator,
                                            is_async: method.function.is_async,
                                        });
                                        
                                        methods.push(HirMethod {
                                            name: m_name,
                                            func_id: f_id,
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                        
                        self.classes.insert(name.clone(), HirClass {
                            name: name.clone(),
                            super_name,
                            fields,
                            methods,
                            getters,
                            setters,
                            static_getters: Vec::new(),
                            static_setters: Vec::new(),
                        });
                        
                        self.current_super = old_super;
                        
                        self.exports.default = Some(binding);
                        self.exports.classes.insert("default".to_string(), name);
                    }
                    _ => {}
                }
                Ok(())
            }
            ModuleDecl::ExportDefaultExpr(export) => {
                let expr = self.lower_expr(&export.expr)?;
                let bid = self.declare("__bs_default_expr");
                out.push(HirStmt::Let {
                    binding: bid,
                    name: "__bs_default_expr".to_string(),
                    init: Some(expr),
                });
                self.exports.default = Some(bid);
                Ok(())
            }
            ModuleDecl::ExportNamed(export) => {
                let src_opt = export.src.as_ref().map(|s| s.value.as_wtf8().to_string_lossy().into_owned());
                for spec in &export.specifiers {
                    if let ExportSpecifier::Named(named_spec) = spec {
                        let local_name = match &named_spec.orig {
                            ModuleExportName::Ident(id) => id.sym.to_string(),
                            ModuleExportName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
                        };
                        let export_name = match &named_spec.exported {
                            Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
                            Some(ModuleExportName::Str(s)) => s.value.as_wtf8().to_string_lossy().into_owned(),
                            None => local_name.clone(),
                        };
                        
                        if let Some(src) = &src_opt {
                            self.exports.re_exports.push(ReExport {
                                src: src.clone(),
                                local: local_name,
                                exported: export_name,
                            });
                        } else {
                            if let Some(bid) = self.lookup(&local_name) {
                                self.exports.named.insert(export_name.clone(), bid);
                                if let Some(f) = self.functions.iter().find(|f| f.name == local_name) {
                                    self.exports.functions.insert(export_name.clone(), f.id);
                                }
                                if self.classes.contains_key(&local_name) {
                                    self.exports.classes.insert(export_name, local_name);
                                }
                            }
                        }
                    } else if let ExportSpecifier::Namespace(ns_spec) = spec {
                        let export_name = match &ns_spec.name {
                            ModuleExportName::Ident(id) => id.sym.to_string(),
                            ModuleExportName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
                        };
                        if let Some(src) = &src_opt {
                            self.exports.re_exports.push(ReExport {
                                src: src.clone(),
                                local: "*".to_string(),
                                exported: export_name,
                            });
                        }
                    }
                }
                Ok(())
            }
            ModuleDecl::ExportAll(export) => {
                let src = export.src.value.as_wtf8().to_string_lossy().into_owned();
                self.exports.export_alls.push(src);
                Ok(())
            }
            _ => Ok(())
        }
    }

}
