use swc_core::ecma::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_decl(&mut self, decl: &Decl, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        self.lower_decl(decl, out)
    }

    pub(crate) fn lower_decl(&mut self, decl: &Decl, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match decl {
            Decl::Var(var) => {
                for d in &var.decls {
                    match &d.name {
                        Pat::Ident(ident) => {
                            let name = ident.sym.to_string();
                            let binding = self.declare(&name);
                            let init = match &d.init {
                                Some(e) => {
                                    if let Expr::Class(ce) = &**e {
                                        Some(self.lower_expr_class_with_name(ce, name.clone())?)
                                    } else {
                                        Some(self.lower_expr(e)?)
                                    }
                                }
                                None => None,
                            };
                            if let Some(HirExpr::Lit(Literal::String(ref s))) = init {
                                self.const_strings.insert(binding, s.clone());
                            }
                            out.push(HirStmt::Let { binding, name, init });
                        }
                        other_pat => {
                            let init_expr = match &d.init {
                                Some(e) => self.lower_expr(e)?,
                                None => HirExpr::Lit(Literal::Undefined),
                            };
                            let temp_name = format!("_temp_destruct_{}", self.fresh_func_id());
                            let temp_binding = self.declare(&temp_name);
                            out.push(HirStmt::Let {
                                binding: temp_binding,
                                name: temp_name.clone(),
                                init: Some(init_expr),
                            });
                            self.lower_pattern(other_pat, HirExpr::Var(temp_binding), out)?;
                        }
                    }
                }
                Ok(())
            }
            Decl::Fn(fn_decl) => {
                let name = fn_decl.ident.sym.to_string();
                let func_id = self.fresh_func_id();
                // Register the function name in the current scope
                let binding = self.declare(&name);

                self.function_stack.push(func_id);

                // Enter a new scope for the function body
                self.push_scope();
                let mut params = Vec::new();
                let mut param_destruct_stmts = Vec::new();
                for (param_idx, p) in fn_decl.function.params.iter().enumerate() {
                    match &p.pat {
                        Pat::Ident(ident) => {
                            let pname = ident.sym.to_string();
                            let pid = self.declare(&pname);
                            params.push((pid, pname));
                        }
                        other_pat => {
                            let pname = format!("_param_{}", param_idx);
                            let pid = self.declare(&pname);
                            params.push((pid, pname.clone()));
                            self.lower_pattern(other_pat, HirExpr::Var(pid), &mut param_destruct_stmts)?;
                        }
                    }
                }
                let body = match &fn_decl.function.body {
                    Some(b) => self.lower_block_stmts(b)?,
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
                    is_generator: fn_decl.function.is_generator,
                    is_async: fn_decl.function.is_async,
                });
                
                if self.function_stack.len() > 1 {
                    // Nested function closure!
                    out.push(HirStmt::Let {
                        binding,
                        name,
                        init: Some(HirExpr::Closure { func_id, captures: Vec::new() }),
                    });
                } else {
                    // Top-level global function
                    out.push(HirStmt::Let {
                        binding,
                        name: name.clone(),
                        init: Some(HirExpr::Closure { func_id, captures: Vec::new() }),
                    });
                    out.push(HirStmt::FuncDecl { id: func_id, name, params, body: full_body });
                }
                Ok(())
            }
            Decl::Class(class_decl) => self.lower_class_decl(class_decl, out),
            _ => Ok(()),
        }
    }

    fn lower_class_decl(&mut self, class_decl: &ClassDecl, _out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let class_name = class_decl.ident.sym.to_string();
        let binding = self.declare(&class_name);
        
        let super_name = class_decl.class.super_class.as_ref().and_then(|expr| {
            if let Expr::Ident(id) = &**expr {
                Some(id.sym.to_string())
            } else {
                None
            }
        });

        let old_super = self.current_super.clone();
        self.current_super = super_name.clone();

        let old_class = self.current_class.take();
        self.current_class = Some((class_name.clone(), binding));

        let mut own_fields = Vec::new();
        let mut methods = Vec::new();
        let mut getters = Vec::new();
        let mut setters = Vec::new();
        let mut static_getters = Vec::new();
        let mut static_setters = Vec::new();
        let mut static_fields = Vec::new();
        let mut static_methods = Vec::new();
        let mut explicit_constructor = None;
        let mut static_blocks = Vec::new();

        // Collect fields first
        for member in &class_decl.class.body {
            match member {
                ClassMember::StaticBlock(sb) => {
                    static_blocks.push(sb.clone());
                }
                ClassMember::ClassProp(prop) => {
                    let name = self.prop_name_to_string(&prop.key);
                    if !prop.is_static {
                        own_fields.push(name);
                    } else {
                        let init = match &prop.value {
                            Some(e) => Some(self.lower_expr(e)?),
                            None => None,
                        };
                        static_fields.push((name, init));
                    }
                }
                ClassMember::Constructor(ctor) => {
                    explicit_constructor = Some(ctor.clone());
                }
                ClassMember::Method(method) => {
                    let base_name = self.prop_name_to_string(&method.key);
                    let name = match method.kind {
                        MethodKind::Getter => {
                            if !method.is_static {
                                getters.push(base_name.clone());
                            } else {
                                static_getters.push(base_name.clone());
                            }
                            format!("__get_{}", base_name)
                        }
                        MethodKind::Setter => {
                            if !method.is_static {
                                setters.push(base_name.clone());
                            } else {
                                static_setters.push(base_name.clone());
                            }
                            format!("__set_{}", base_name)
                        }
                        MethodKind::Method => base_name,
                    };
                    let func_id = self.fresh_func_id();
                    if !method.is_static {
                        methods.push(HirMethod { name, func_id });
                    } else {
                        static_methods.push(HirMethod { name, func_id });
                    }
                }
                ClassMember::PrivateProp(prop) => {
                    let name = format!("__private_{}", prop.key.name);
                    if !prop.is_static {
                        own_fields.push(name);
                    } else {
                        let init = match &prop.value {
                            Some(e) => Some(self.lower_expr(e)?),
                            None => None,
                        };
                        static_fields.push((name, init));
                    }
                }
                ClassMember::PrivateMethod(method) => {
                    let base_name = format!("__private_{}", method.key.name);
                    let name = match method.kind {
                        MethodKind::Getter => {
                            if !method.is_static {
                                getters.push(base_name.clone());
                            } else {
                                static_getters.push(base_name.clone());
                            }
                            format!("__get_{}", base_name)
                        }
                        MethodKind::Setter => {
                            if !method.is_static {
                                setters.push(base_name.clone());
                            } else {
                                static_setters.push(base_name.clone());
                            }
                            format!("__set_{}", base_name)
                        }
                        MethodKind::Method => base_name,
                    };
                    let func_id = self.fresh_func_id();
                    if !method.is_static {
                        methods.push(HirMethod { name, func_id });
                    } else {
                        static_methods.push(HirMethod { name, func_id });
                    }
                }
                _ => {}
            }
        }

        // Lower constructor
        let _ctor_func_id = if let Some(ctor) = explicit_constructor {
            let func_id = self.fresh_func_id();
            let name = format!("__bs_class_{}_constructor", class_name);
            
            self.function_stack.push(func_id);
            self.push_scope();
            let this_id = self.declare("this");
            let old_this = self.this_binding;
            self.this_binding = Some(this_id);
            
            let mut params = vec![(this_id, "this".to_string())];
            let mut param_destruct_stmts = Vec::new();
            for (param_idx, p) in ctor.params.iter().enumerate() {
                if let ParamOrTsParamProp::Param(param) = p {
                    match &param.pat {
                        Pat::Ident(ident) => {
                            let pname = ident.sym.to_string();
                            let pid = self.declare(&pname);
                            params.push((pid, pname));
                        }
                        other_pat => {
                            let pname = format!("_param_{}", param_idx);
                            let pid = self.declare(&pname);
                            params.push((pid, pname.clone()));
                            self.lower_pattern(other_pat, HirExpr::Var(pid), &mut param_destruct_stmts)?;
                        }
                    }
                }
            }

            let body = match &ctor.body {
                Some(b) => self.lower_block_stmts(b)?,
                None => Vec::new(),
            };
            let mut full_body = param_destruct_stmts;
            full_body.extend(body);

            self.this_binding = old_this;
            self.pop_scope();
            self.function_stack.pop();

            self.functions.push(HirFunction {
                id: func_id,
                name: name.clone(),
                params,
                body: full_body,
                captures: Vec::new(),
                is_generator: false,
                is_async: false,
            });
            func_id
        } else {
            // Synthesize default constructor
            let func_id = self.fresh_func_id();
            let name = format!("__bs_class_{}_constructor", class_name);
            
            self.function_stack.push(func_id);
            self.push_scope();
            let this_id = self.declare("this");
            let old_this = self.this_binding;
            self.this_binding = Some(this_id);
            
            let params = vec![(this_id, "this".to_string())];
            let mut body = Vec::new();
            
            // If has super class, default ctor calls super constructor with this
            if let Some(super_c) = &super_name {
                body.push(HirStmt::Expr(HirExpr::Call {
                    callee: Box::new(HirExpr::GlobalRef(format!("__bs_class_{}_constructor", super_c))),
                    args: vec![HirExpr::Var(this_id)],
                }));
            }
            
            self.this_binding = old_this;
            self.pop_scope();
            self.function_stack.pop();

            self.functions.push(HirFunction {
                id: func_id,
                name,
                params,
                body,
                captures: Vec::new(),
                is_generator: false,
                is_async: false,
            });
            func_id
        };

        // Lower methods
        for member in &class_decl.class.body {
            let (is_static, key_id_sym, method_kind, function) = match member {
                ClassMember::Method(m) => (m.is_static, self.prop_name_to_string(&m.key), m.kind, &m.function),
                ClassMember::PrivateMethod(m) => (m.is_static, format!("__private_{}", m.key.name), m.kind, &m.function),
                _ => continue,
            };
            if !is_static {
                let base_name = key_id_sym;
                let m_name = match method_kind {
                    MethodKind::Getter => format!("__get_{}", base_name),
                    MethodKind::Setter => format!("__set_{}", base_name),
                    MethodKind::Method => base_name,
                };
                let m_func = methods.iter().find(|m| m.name == m_name).unwrap();
                let func_name = format!("__bs_class_{}_{}", class_name, m_name);

                self.function_stack.push(m_func.func_id);
                self.push_scope();
                let this_id = self.declare("this");
                let old_this = self.this_binding;
                self.this_binding = Some(this_id);

                let mut params = vec![(this_id, "this".to_string())];
                let mut param_destruct_stmts = Vec::new();
                for (param_idx, p) in function.params.iter().enumerate() {
                    match &p.pat {
                        Pat::Ident(ident) => {
                            let pname = ident.sym.to_string();
                            let pid = self.declare(&pname);
                            params.push((pid, pname));
                        }
                        other_pat => {
                            let pname = format!("_param_{}", param_idx);
                            let pid = self.declare(&pname);
                            params.push((pid, pname.clone()));
                            self.lower_pattern(other_pat, HirExpr::Var(pid), &mut param_destruct_stmts)?;
                        }
                    }
                }

                let body = match &function.body {
                    Some(b) => self.lower_block_stmts(b)?,
                    None => Vec::new(),
                };
                let mut full_body = param_destruct_stmts;
                full_body.extend(body);

                self.this_binding = old_this;
                self.pop_scope();
                self.function_stack.pop();

                self.functions.push(HirFunction {
                    id: m_func.func_id,
                    name: func_name,
                    params,
                    body: full_body,
                    captures: Vec::new(),
                    is_generator: function.is_generator,
                    is_async: function.is_async,
                });
            }
        }

        // Lower static methods
        for member in &class_decl.class.body {
            let (is_static, key_id_sym, method_kind, function) = match member {
                ClassMember::Method(m) => (m.is_static, self.prop_name_to_string(&m.key), m.kind, &m.function),
                ClassMember::PrivateMethod(m) => (m.is_static, format!("__private_{}", m.key.name), m.kind, &m.function),
                _ => continue,
            };
            if is_static {
                let base_name = key_id_sym;
                let m_name = match method_kind {
                    MethodKind::Getter => format!("__get_{}", base_name),
                    MethodKind::Setter => format!("__set_{}", base_name),
                    MethodKind::Method => base_name,
                };
                let m_func = static_methods.iter().find(|m| m.name == m_name).unwrap();
                let func_name = format!("__bs_class_{}_static_{}", class_name, m_name);

                self.function_stack.push(m_func.func_id);
                self.push_scope();

                let mut params = Vec::new();
                let mut param_destruct_stmts = Vec::new();
                for (param_idx, p) in function.params.iter().enumerate() {
                    match &p.pat {
                        Pat::Ident(ident) => {
                            let pname = ident.sym.to_string();
                            let pid = self.declare(&pname);
                            params.push((pid, pname));
                        }
                        other_pat => {
                            let pname = format!("_param_{}", param_idx);
                            let pid = self.declare(&pname);
                            params.push((pid, pname.clone()));
                            self.lower_pattern(other_pat, HirExpr::Var(pid), &mut param_destruct_stmts)?;
                        }
                    }
                }

                let body = match &function.body {
                    Some(b) => self.lower_block_stmts(b)?,
                    None => Vec::new(),
                };
                let mut full_body = param_destruct_stmts;
                full_body.extend(body);

                self.pop_scope();
                self.function_stack.pop();

                self.functions.push(HirFunction {
                    id: m_func.func_id,
                    name: func_name,
                    params,
                    body: full_body,
                    captures: Vec::new(),
                    is_generator: function.is_generator,
                    is_async: function.is_async,
                });
            }
        }

        // Declare the class constructor variable
        let class_obj = HirExpr::Call {
            callee: Box::new(HirExpr::GlobalRef("__bs_new_object".to_string())),
            args: vec![],
        };
        _out.push(HirStmt::Let {
            binding,
            name: class_name.clone(),
            init: Some(class_obj),
        });

        // Assign static methods to constructor object
        for m in &static_methods {
            let closure_expr = HirExpr::Closure {
                func_id: m.func_id,
                captures: Vec::new(),
            };
            let assign_expr = HirExpr::MemberSet {
                object: Box::new(HirExpr::Var(binding)),
                property: m.name.clone(),
                value: Box::new(closure_expr),
            };
            _out.push(HirStmt::Expr(assign_expr));
        }

        // Assign static fields to constructor object
        for (f_name, f_init) in static_fields {
            let init_val = f_init.unwrap_or(HirExpr::Lit(Literal::Undefined));
            let assign_expr = HirExpr::MemberSet {
                object: Box::new(HirExpr::Var(binding)),
                property: f_name,
                value: Box::new(init_val),
            };
            _out.push(HirStmt::Expr(assign_expr));
        }

        // Run static initialization blocks
        let old_this = self.this_binding;
        self.this_binding = Some(binding);
        for sb in &static_blocks {
            let stmts = self.lower_block_stmts(&sb.body)?;
            _out.extend(stmts);
        }
        self.this_binding = old_this;

        self.current_super = old_super;
        self.current_class = old_class;

        // Register class metadata
        self.classes.insert(
            class_name.clone(),
            HirClass {
                name: class_name.clone(),
                super_name,
                fields: own_fields,
                methods,
                getters,
                setters,
                static_getters,
                static_setters,
            },
        );

        Ok(())
    }

}
