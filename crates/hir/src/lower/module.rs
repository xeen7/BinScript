//! Module-level lowering: `lower_module` and `lower_module_decl`.

use oxc::ast::ast::*;
use diagnostics::CompileResult;
use crate::types::*;
use super::context::LowerCtx;
use super::capture::populate_closure_captures;

impl LowerCtx {
    pub(crate) fn lower_module(&mut self, program: &Program) -> CompileResult<HirModule> {
        // Pre-collect all function, method, and constructor names in the module
        for stmt in &program.body {
            match stmt {
                Statement::FunctionDeclaration(fn_decl) => {
                    if let Some(id) = &fn_decl.id {
                        self.function_names.insert(id.name.to_string());
                    }
                }
                Statement::ClassDeclaration(class_decl) => {
                    if let Some(id) = &class_decl.id {
                        let class_name = id.name.to_string();
                        self.function_names.insert(format!("__bs_class_{}_constructor", class_name));
                        for member in &class_decl.body.body {
                            if let ClassElement::MethodDefinition(method) = member {
                                if !method.r#static {
                                    let m_name = match &method.key {
                                        PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                        PropertyKey::StringLiteral(s) => s.value.to_string(),
                                        PropertyKey::NumericLiteral(n) => n.value.to_string(),
                                        _ => "unknown".to_string(),
                                    };
                                    self.function_names.insert(format!("__bs_class_{}_{}", class_name, m_name));
                                }
                            }
                        }
                    }
                }
                Statement::ExportNamedDeclaration(export) => {
                    if let Some(decl) = &export.declaration {
                        match decl {
                            Declaration::FunctionDeclaration(fn_decl) => {
                                if let Some(id) = &fn_decl.id {
                                    self.function_names.insert(id.name.to_string());
                                }
                            }
                            Declaration::ClassDeclaration(class_decl) => {
                                if let Some(id) = &class_decl.id {
                                    let class_name = id.name.to_string();
                                    self.function_names.insert(format!("__bs_class_{}_constructor", class_name));
                                    for member in &class_decl.body.body {
                                        if let ClassElement::MethodDefinition(method) = member {
                                            if !method.r#static {
                                                let m_name = match &method.key {
                                                    PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                                    PropertyKey::StringLiteral(s) => s.value.to_string(),
                                                    PropertyKey::NumericLiteral(n) => n.value.to_string(),
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
                Statement::ExportDefaultDeclaration(export) => {
                    match &export.declaration {
                        ExportDefaultDeclarationKind::FunctionDeclaration(fn_decl) => {
                            if let Some(id) = &fn_decl.id {
                                self.function_names.insert(id.name.to_string());
                            }
                        }
                        ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                            if let Some(id) = &class_decl.id {
                                let class_name = id.name.to_string();
                                self.function_names.insert(format!("__bs_class_{}_constructor", class_name));
                                for member in &class_decl.body.body {
                                    if let ClassElement::MethodDefinition(method) = member {
                                        if !method.r#static {
                                            let m_name = match &method.key {
                                                PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                                PropertyKey::StringLiteral(s) => s.value.to_string(),
                                                PropertyKey::NumericLiteral(n) => n.value.to_string(),
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
                _ => {}
            }
        }

        let mut stmts = Vec::new();
        for stmt in &program.body {
            match stmt {
                Statement::ImportDeclaration(_) |
                Statement::ExportNamedDeclaration(_) |
                Statement::ExportDefaultDeclaration(_) |
                Statement::ExportAllDeclaration(_) => {
                    self.lower_module_decl(stmt, &mut stmts)?;
                }
                _ => {
                    self.lower_stmt(stmt, &mut stmts)?;
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

    pub(crate) fn lower_module_decl(&mut self, decl: &Statement, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match decl {
            Statement::ImportDeclaration(_) => {
                // Pre-resolved and injected by caller
                Ok(())
            }
            Statement::ExportNamedDeclaration(export) => {
                if let Some(d) = &export.declaration {
                    self.lower_decl(d, out)?;
                    
                    match d {
                        Declaration::VariableDeclaration(var_decl) => {
                            for decl in &var_decl.declarations {
                                if let BindingPattern::BindingIdentifier(binding_ident) = &decl.id {
                                    let name = binding_ident.name.to_string();
                                    if let Some(bid) = self.lookup(&name) {
                                        self.exports.named.insert(name, bid);
                                    }
                                }
                            }
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            if let Some(id) = &fn_decl.id {
                                let name = id.name.to_string();
                                if let Some(bid) = self.lookup(&name) {
                                    self.exports.named.insert(name.clone(), bid);
                                }
                                if let Some(f) = self.functions.iter().find(|f| f.name == name) {
                                    self.exports.functions.insert(name, f.id);
                                }
                            }
                        }
                        Declaration::ClassDeclaration(class_decl) => {
                            if let Some(id) = &class_decl.id {
                                let name = id.name.to_string();
                                if let Some(bid) = self.lookup(&name) {
                                    self.exports.named.insert(name.clone(), bid);
                                }
                                self.exports.classes.insert(name.clone(), name);
                            }
                        }
                        _ => {}
                    }
                } else {
                    let src_opt = export.source.as_ref().map(|s| s.value.to_string());
                    for spec in &export.specifiers {
                        let local_name = match &spec.local {
                            ModuleExportName::IdentifierName(id) => id.name.to_string(),
                            ModuleExportName::IdentifierReference(id) => id.name.to_string(),
                            ModuleExportName::StringLiteral(s) => s.value.to_string(),
                        };
                        let export_name = match &spec.exported {
                            ModuleExportName::IdentifierName(id) => id.name.to_string(),
                            ModuleExportName::IdentifierReference(id) => id.name.to_string(),
                            ModuleExportName::StringLiteral(s) => s.value.to_string(),
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
                    }
                }
                Ok(())
            }
            Statement::ExportDefaultDeclaration(export) => {
                match &export.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(fn_decl) => {
                        let name = fn_decl.id.as_ref()
                            .map(|id| id.name.to_string())
                            .unwrap_or_else(|| "__bs_default_fn".to_string());
                        
                        let func_id = self.fresh_func_id();
                        let binding = self.declare(&name);
                        
                        self.function_stack.push(func_id);
                        self.push_scope();
                        let mut params = Vec::new();
                        let mut param_destruct_stmts = Vec::new();
                        self.lower_formal_parameters(&fn_decl.params, &mut params, &mut param_destruct_stmts)?;

                        let body = match &fn_decl.body {
                            Some(b) => self.lower_function_body(b)?,
                            None => Vec::new(),
                        };
                        let mut full_body = param_destruct_stmts;
                        full_body.extend(body);

                        self.pop_scope();
                        self.function_stack.pop();
                        
                        self.functions.push(HirFunction {
                            id: func_id,
                            name: name.clone(),
                            params: params.clone(),
                            body: full_body.clone(),
                            captures: Vec::new(),
                            is_generator: fn_decl.generator,
                            is_async: fn_decl.r#async,
                        });
                        
                        out.push(HirStmt::FuncDecl {
                            id: func_id,
                            name: name.clone(),
                            params,
                            body: full_body,
                        });
                        
                        self.exports.default = Some(binding);
                        self.exports.functions.insert("default".to_string(), func_id);
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(class_decl) => {
                        let name = class_decl.id.as_ref()
                            .map(|id| id.name.to_string())
                            .unwrap_or_else(|| "__bs_default_class".to_string());
                        
                        let binding = self.declare(&name);
                        let super_name = class_decl.super_class.as_ref().and_then(|expr| {
                            if let Expression::Identifier(id) = expr {
                                Some(id.name.to_string())
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
                        
                        for member in &class_decl.body.body {
                            match member {
                                ClassElement::PropertyDefinition(prop) => {
                                    if let PropertyKey::StaticIdentifier(id) = &prop.key {
                                        fields.push(id.name.to_string());
                                    }
                                }
                                ClassElement::MethodDefinition(method) => {
                                    if !method.r#static {
                                        let base_name = match &method.key {
                                            PropertyKey::StaticIdentifier(id) => id.name.to_string(),
                                            PropertyKey::StringLiteral(s) => s.value.to_string(),
                                            PropertyKey::NumericLiteral(n) => n.value.to_string(),
                                            _ => "unknown".to_string(),
                                        };
                                        let m_name = match method.kind {
                                            MethodDefinitionKind::Get => {
                                                getters.push(base_name.clone());
                                                format!("__get_{}", base_name)
                                            }
                                            MethodDefinitionKind::Set => {
                                                setters.push(base_name.clone());
                                                format!("__set_{}", base_name)
                                            }
                                            MethodDefinitionKind::Method => base_name,
                                            _ => "unknown".to_string(),
                                        };
                                        let f_id = self.fresh_func_id();
                                        
                                        self.function_stack.push(f_id);
                                        self.push_scope();
                                        
                                        let old_this = self.this_binding;
                                        let this_id = self.declare("this");
                                        self.this_binding = Some(this_id);
                                        
                                        let mut params = Vec::new();
                                        params.push((this_id, "this".to_string()));
                                        
                                        for p in &method.value.params.items {
                                            if let BindingPattern::BindingIdentifier(ident) = &p.pattern {
                                                let pname = ident.name.to_string();
                                                let pid = self.declare(&pname);
                                                params.push((pid, pname));
                                            }
                                        }
                                        
                                        let f_body = match &method.value.body {
                                            Some(b) => self.lower_function_body(b)?,
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
                                            is_generator: method.value.generator,
                                            is_async: method.value.r#async,
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
                    _ => {
                        let expr = export.declaration.as_expression().unwrap();
                        let expr = self.lower_expr(expr)?;
                        let bid = self.declare("__bs_default_expr");
                        out.push(HirStmt::Let {
                            binding: bid,
                            name: "__bs_default_expr".to_string(),
                            init: Some(expr),
                        });
                        self.exports.default = Some(bid);
                    }
                }
                Ok(())
            }
            Statement::ExportAllDeclaration(export) => {
                let src = export.source.value.to_string();
                self.exports.export_alls.push(src);
                Ok(())
            }
            _ => Ok(())
        }
    }

}
