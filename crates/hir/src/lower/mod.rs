//! SWC AST → HIR lowering.
//!
//! Walks the SWC JavaScript AST (after TS type stripping) and produces
//! a simplified HIR suitable for further lowering to MIR.

use std::collections::{HashMap, HashSet};

use swc_core::ecma::ast::*;

use diagnostics::CompileResult;

use crate::types::*;

mod expr;
mod stmt;

/// Lowers an SWC `Module` into an `HirModule`.
pub fn lower_module(module: &Module) -> CompileResult<HirModule> {
    let mut ctx = LowerCtx::new();
    ctx.lower_module(module)
}

/// Lower a module with pre-resolved import bindings injected into scope.
pub fn lower_module_with_imports(
    module: &Module,
    import_bindings: std::collections::HashMap<String, BindingId>,
    import_functions: std::collections::HashMap<String, String>,
    import_classes: std::collections::HashMap<String, String>,
    starting_binding_id: BindingId,
    starting_func_id: FuncId,
) -> CompileResult<HirModule> {
    let mut ctx = LowerCtx::new();
    ctx.next_binding = starting_binding_id;
    ctx.next_func = starting_func_id;
    
    // Inject import bindings into top-level scope
    for (name, bid) in import_bindings {
        ctx.scopes[0].insert(name, bid);
    }
    
    ctx.function_aliases = import_functions;
    ctx.class_aliases = import_classes;
    
    ctx.lower_module(module)
}

// ===========================================================================
// Internal lowering context
// ===========================================================================

pub(crate) struct LowerCtx {
    pub(crate) next_binding: BindingId,
    pub(crate) next_func: FuncId,
    /// Stack of scopes. Each scope maps names → BindingId.
    pub(crate) scopes: Vec<HashMap<String, BindingId>>,
    /// All function bodies collected during lowering.
    pub(crate) functions: Vec<HirFunction>,
    /// Pre-collected set of all function names in the module.
    pub(crate) function_names: std::collections::HashSet<String>,
    // --- Stage 2 additions ---
    pub(crate) classes: HashMap<String, HirClass>,
    pub(crate) this_binding: Option<BindingId>,
    pub(crate) current_super: Option<String>,
    // --- Stage 3 additions ---
    pub(crate) binding_owners: HashMap<BindingId, FuncId>,
    pub(crate) function_stack: Vec<FuncId>,
    pub(crate) func_captures: HashMap<FuncId, HashSet<BindingId>>,
    pub(crate) reassigned_bindings: HashSet<BindingId>,
    pub(crate) const_strings: HashMap<BindingId, String>,
    pub(crate) exports: ModuleExports,
    pub(crate) function_aliases: HashMap<String, String>,
    pub(crate) class_aliases: HashMap<String, String>,
    /// Tracks the class currently being lowered: (class_name, class_binding_id)
    pub(crate) current_class: Option<(String, BindingId)>,
}

impl LowerCtx {
    fn new() -> Self {
        Self {
            next_binding: 0,
            next_func: 1,
            scopes: vec![HashMap::new()], // global scope
            functions: Vec::new(),
            function_names: std::collections::HashSet::new(),
            classes: HashMap::new(),
            this_binding: None,
            current_super: None,
            binding_owners: HashMap::new(),
            function_stack: vec![0], // main module is func 0
            func_captures: HashMap::new(),
            reassigned_bindings: HashSet::new(),
            const_strings: HashMap::new(),
            exports: ModuleExports::default(),
            function_aliases: HashMap::new(),
            class_aliases: HashMap::new(),
            current_class: None,
        }
    }

    // ── scope helpers ──────────────────────────────────────────────────────

    pub(crate) fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub(crate) fn declare(&mut self, name: &str) -> BindingId {
        let id = self.next_binding;
        self.next_binding += 1;
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), id);
        }
        let current_func = *self.function_stack.last().unwrap_or(&0);
        self.binding_owners.insert(id, current_func);
        id
    }

    pub(crate) fn insert_binding(&mut self, name: String, id: BindingId) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, id);
        }
    }

    pub(crate) fn lookup(&self, name: &str) -> Option<BindingId> {
        for scope in self.scopes.iter().rev() {
            if let Some(&id) = scope.get(name) {
                return Some(id);
            }
        }
        None
    }

    pub(crate) fn record_lookup(&mut self, bid: BindingId) {
        if let Some(&defining_func) = self.binding_owners.get(&bid) {
            if let Some(&current_func) = self.function_stack.last() {
                if current_func != defining_func {
                    let mut i = self.function_stack.len() - 1;
                    while i > 0 {
                        let f = self.function_stack[i];
                        if f == defining_func {
                            break;
                        }
                        self.func_captures.entry(f).or_default().insert(bid);
                        i -= 1;
                    }
                }
            }
        }
    }

    pub(crate) fn fresh_func_id(&mut self) -> FuncId {
        let id = self.next_func;
        self.next_func += 1;
        id
    }

    // ── module ─────────────────────────────────────────────────────────────

    fn lower_module(&mut self, module: &Module) -> CompileResult<HirModule> {
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

    fn lower_module_decl(&mut self, decl: &ModuleDecl, out: &mut Vec<HirStmt>) -> CompileResult<()> {
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

    pub(crate) fn prop_name_to_string(&self, prop: &PropName) -> String {
        match prop {
            PropName::Ident(id) => id.sym.to_string(),
            PropName::Str(s) => s.value.as_wtf8().to_string_lossy().into_owned(),
            PropName::Num(n) => n.value.to_string(),
            _ => "unknown".to_string(),
        }
    }

    pub(crate) fn lower_pattern(&mut self, pat: &Pat, base_expr: HirExpr, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match pat {
            Pat::Ident(binding_ident) => {
                let name = binding_ident.sym.to_string();
                let binding = self.declare(&name);
                out.push(HirStmt::Let {
                    binding,
                    name,
                    init: Some(base_expr),
                });
                Ok(())
            }
            Pat::Array(array_pat) => {
                for (idx, elem_opt) in array_pat.elems.iter().enumerate() {
                    if let Some(elem_pat) = elem_opt {
                        if let Pat::Rest(rest_pat) = elem_pat {
                            let rest_expr = HirExpr::MethodCall {
                                object: Box::new(base_expr.clone()),
                                method: "slice".to_string(),
                                args: vec![
                                    HirExpr::Lit(Literal::Number(idx as f64)),
                                    HirExpr::Lit(Literal::Undefined),
                                ],
                            };
                            self.lower_pattern(&rest_pat.arg, rest_expr, out)?;
                        } else {
                            let elem_expr = HirExpr::IndexGet {
                                object: Box::new(base_expr.clone()),
                                index: Box::new(HirExpr::Lit(Literal::Number(idx as f64))),
                            };
                            self.lower_pattern(elem_pat, elem_expr, out)?;
                        }
                    }
                }
                Ok(())
            }
            Pat::Object(object_pat) => {
                for prop in &object_pat.props {
                    match prop {
                        ObjectPatProp::Assign(assign_prop) => {
                            let prop_name = assign_prop.key.sym.to_string();
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(base_expr.clone()),
                                property: prop_name.clone(),
                            };
                            let val_expr = if let Some(default_expr) = &assign_prop.value {
                                let default_val = self.lower_expr(default_expr)?;
                                let cond = HirExpr::BinOp(
                                    BinOp::Eq,
                                    Box::new(member_expr.clone()),
                                    Box::new(HirExpr::Lit(Literal::Undefined)),
                                );
                                HirExpr::Ternary {
                                    cond: Box::new(cond),
                                    then_expr: Box::new(default_val),
                                    else_expr: Box::new(member_expr),
                                }
                            } else {
                                member_expr
                            };
                            let binding = self.declare(&prop_name);
                            out.push(HirStmt::Let {
                                binding,
                                name: prop_name,
                                init: Some(val_expr),
                            });
                        }
                        ObjectPatProp::KeyValue(kv_prop) => {
                            let prop_name = self.prop_name_to_string(&kv_prop.key);
                            let member_expr = HirExpr::MemberGet {
                                object: Box::new(base_expr.clone()),
                                property: prop_name,
                            };
                            self.lower_pattern(&kv_prop.value, member_expr, out)?;
                        }
                        ObjectPatProp::Rest(rest_pat) => {
                            let mut extracted_keys = Vec::new();
                            for other_prop in &object_pat.props {
                                match other_prop {
                                    ObjectPatProp::Assign(ap) => {
                                        extracted_keys.push(HirExpr::Lit(Literal::String(ap.key.sym.to_string())));
                                    }
                                    ObjectPatProp::KeyValue(kv) => {
                                        extracted_keys.push(HirExpr::Lit(Literal::String(self.prop_name_to_string(&kv.key))));
                                    }
                                    ObjectPatProp::Rest(_) => {}
                                }
                            }
                            let keys_array = HirExpr::ArrayLit(extracted_keys);
                            let rest_expr = HirExpr::Call {
                                callee: Box::new(HirExpr::GlobalRef("__bs_object_rest".to_string())),
                                args: vec![base_expr.clone(), keys_array],
                            };
                            self.lower_pattern(&rest_pat.arg, rest_expr, out)?;
                        }
                    }
                }
                Ok(())
            }
            Pat::Assign(assign_pat) => {
                let default_val = self.lower_expr(&assign_pat.right)?;
                let cond = HirExpr::BinOp(
                    BinOp::Eq,
                    Box::new(base_expr.clone()),
                    Box::new(HirExpr::Lit(Literal::Undefined)),
                );
                let final_expr = HirExpr::Ternary {
                    cond: Box::new(cond),
                    then_expr: Box::new(default_val),
                    else_expr: Box::new(base_expr),
                };
                self.lower_pattern(&assign_pat.left, final_expr, out)?;
                Ok(())
            }
            _ => Err(diagnostics::CompileError::Lowering {
                message: "Unsupported destructuring pattern type".into(),
            }),
        }
    }
}

// ===========================================================================
// Operator conversion
// ===========================================================================

pub(crate) fn conv_bin_op(op: BinaryOp) -> BinOp {
    match op {
        BinaryOp::Exp => BinOp::Exp,
        BinaryOp::NullishCoalescing => BinOp::NullishCoalescing,
        BinaryOp::In => BinOp::In,
        BinaryOp::Add => BinOp::Add,
        BinaryOp::Sub => BinOp::Sub,
        BinaryOp::Mul => BinOp::Mul,
        BinaryOp::Div => BinOp::Div,
        BinaryOp::Mod => BinOp::Mod,
        BinaryOp::EqEq => BinOp::Eq,
        BinaryOp::NotEq => BinOp::Ne,
        BinaryOp::EqEqEq => BinOp::StrictEq,
        BinaryOp::NotEqEq => BinOp::StrictNe,
        BinaryOp::Lt => BinOp::Lt,
        BinaryOp::LtEq => BinOp::Le,
        BinaryOp::Gt => BinOp::Gt,
        BinaryOp::GtEq => BinOp::Ge,
        BinaryOp::LogicalAnd => BinOp::And,
        BinaryOp::LogicalOr => BinOp::Or,
        BinaryOp::BitAnd => BinOp::BitAnd,
        BinaryOp::BitOr => BinOp::BitOr,
        BinaryOp::BitXor => BinOp::BitXor,
        BinaryOp::LShift => BinOp::Shl,
        BinaryOp::RShift => BinOp::Shr,
        BinaryOp::ZeroFillRShift => BinOp::UShr,
        _ => BinOp::Add, // fallback for unsupported ops
    }
}

pub(crate) fn conv_unary_op(op: swc_core::ecma::ast::UnaryOp) -> crate::types::UnaryOp {
    match op {
        swc_core::ecma::ast::UnaryOp::Plus => crate::types::UnaryOp::Plus,
        swc_core::ecma::ast::UnaryOp::Minus => crate::types::UnaryOp::Neg,
        swc_core::ecma::ast::UnaryOp::Bang => crate::types::UnaryOp::Not,
        swc_core::ecma::ast::UnaryOp::Tilde => crate::types::UnaryOp::BitNot,
        swc_core::ecma::ast::UnaryOp::TypeOf => crate::types::UnaryOp::Typeof,
        swc_core::ecma::ast::UnaryOp::Void => crate::types::UnaryOp::Void,
        swc_core::ecma::ast::UnaryOp::Delete => crate::types::UnaryOp::Void, // Delete is a unary op in AST but handled differently
        _ => crate::types::UnaryOp::Neg,
    }
}

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
        HirExpr::Spread(inner) => {
            walk_expr(inner, func_captures);
        }
        HirExpr::DeleteProp { object, property } => {
            walk_expr(object, func_captures);
            walk_expr(property, func_captures);
        }
    }
}
