use oxc::ast::ast::*;

use diagnostics::CompileResult;
use crate::types::*;
use crate::lower::LowerCtx;

impl LowerCtx {
    pub(super) fn lower_stmt_decl(&mut self, decl: &Declaration, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        self.lower_decl(decl, out)
    }

    pub(crate) fn lower_decl(&mut self, decl: &Declaration, out: &mut Vec<HirStmt>) -> CompileResult<()> {
        match decl {
            Declaration::VariableDeclaration(var) => {
                for d in &var.declarations {
                    match &d.id {
                        BindingPattern::BindingIdentifier(ident) => {
                            let name = ident.name.to_string();
                            let binding = self.declare(&name);
                            let init = match &d.init {
                                Some(e) => {
                                    if let Expression::ClassExpression(ce) = e {
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
            Declaration::FunctionDeclaration(fn_decl) => {
                let name = fn_decl.id.as_ref().map(|id| id.name.to_string()).unwrap_or_default();
                let func_id = self.fresh_func_id();
                // Register the function name in the current scope
                let binding = self.declare(&name);

                self.function_stack.push(func_id);

                // Enter a new scope for the function body
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
            Declaration::ClassDeclaration(class_decl) => self.lower_class_decl(class_decl, None, out),
            _ => Ok(()),
        }
    }

    pub(crate) fn lower_class_decl(&mut self, class_decl: &Class, name_override: Option<String>, _out: &mut Vec<HirStmt>) -> CompileResult<()> {
        let class_name = name_override.unwrap_or_else(|| class_decl.id.as_ref().map(|id| id.name.to_string()).unwrap_or_default());
        let binding = self.declare(&class_name);
        
        let super_name = class_decl.super_class.as_ref().and_then(|expr| {
            if let Expression::Identifier(id) = expr {
                Some(id.name.to_string())
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
        let mut instance_field_inits: Vec<(String, Option<HirExpr>)> = Vec::new();

        // Collect fields first
        for member in &class_decl.body.body {
            match member {
                ClassElement::StaticBlock(sb) => {
                    static_blocks.push(sb);
                }
                ClassElement::PropertyDefinition(prop) => {
                    let name = self.prop_name_to_string(&prop.key);
                    if !prop.r#static {
                        let init = match &prop.value {
                            Some(e) => Some(self.lower_expr(e)?),
                            None => None,
                        };
                        
                        let field_type = if let Some(type_ann) = &prop.type_annotation {
                            match &type_ann.type_annotation {
                                oxc::ast::ast::TSType::TSNumberKeyword(_) |
                                oxc::ast::ast::TSType::TSStringKeyword(_) |
                                oxc::ast::ast::TSType::TSBooleanKeyword(_) => HirType::Primitive,
                                oxc::ast::ast::TSType::TSTypeReference(ref_type) => {
                                    if let oxc::ast::ast::TSTypeName::IdentifierReference(ident) = &ref_type.type_name {
                                        HirType::Object(ident.name.to_string())
                                    } else {
                                        HirType::Any
                                    }
                                }
                                _ => HirType::Any,
                            }
                        } else {
                            HirType::Any
                        };
                        
                        own_fields.push((name.clone(), field_type));
                        instance_field_inits.push((name, init));
                    } else {
                        let init = match &prop.value {
                            Some(e) => Some(self.lower_expr(e)?),
                            None => None,
                        };
                        static_fields.push((name, init));
                    }
                }
                ClassElement::AccessorProperty(acc) => {
                    let base_name = self.prop_name_to_string(&acc.key);
                    let private_name = format!("__private_{}", base_name);
                    let get_name = format!("__get_{}", base_name);
                    let set_name = format!("__set_{}", base_name);
                    let get_func_id = self.fresh_func_id();
                    let set_func_id = self.fresh_func_id();

                    let init = match &acc.value {
                        Some(e) => Some(self.lower_expr(e)?),
                        None => None,
                    };

                    if !acc.r#static {
                        own_fields.push((private_name.clone(), HirType::Any));
                        instance_field_inits.push((private_name, init));

                        getters.push(base_name.clone());
                        setters.push(base_name.clone());
                        methods.push(HirMethod { name: get_name, func_id: get_func_id });
                        methods.push(HirMethod { name: set_name, func_id: set_func_id });
                    } else {
                        static_fields.push((private_name, init));

                        static_getters.push(base_name.clone());
                        static_setters.push(base_name.clone());
                        static_methods.push(HirMethod { name: get_name, func_id: get_func_id });
                        static_methods.push(HirMethod { name: set_name, func_id: set_func_id });
                    }
                }
                ClassElement::MethodDefinition(method) => {
                    if method.kind == MethodDefinitionKind::Constructor {
                        explicit_constructor = Some(method);
                    } else {
                        let base_name = self.prop_name_to_string(&method.key);
                        let name = match method.kind {
                            MethodDefinitionKind::Get => {
                                if !method.r#static {
                                    getters.push(base_name.clone());
                                } else {
                                    static_getters.push(base_name.clone());
                                }
                                format!("__get_{}", base_name)
                            }
                            MethodDefinitionKind::Set => {
                                if !method.r#static {
                                    setters.push(base_name.clone());
                                } else {
                                    static_setters.push(base_name.clone());
                                }
                                format!("__set_{}", base_name)
                            }
                            MethodDefinitionKind::Method => base_name,
                            _ => base_name, // Constructor handled above
                        };
                        let func_id = self.fresh_func_id();
                        if !method.r#static {
                            methods.push(HirMethod { name, func_id });
                        } else {
                            static_methods.push(HirMethod { name, func_id });
                        }
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
            self.lower_formal_parameters(&ctor.value.params, &mut params, &mut param_destruct_stmts)?;

            let body = match &ctor.value.body {
                Some(b) => self.lower_function_body(b)?,
                None => Vec::new(),
            };
            // Inject instance field initializers at the start of the constructor body
            let mut field_init_stmts = Vec::new();
            for (f_name, f_init) in &instance_field_inits {
                let init_val = f_init.clone().unwrap_or(HirExpr::Lit(Literal::Undefined));
                field_init_stmts.push(HirStmt::Expr(HirExpr::MemberSet {
                    object: Box::new(HirExpr::Var(this_id)),
                    property: f_name.clone(),
                    value: Box::new(init_val),
                }));
            }
            let mut full_body = param_destruct_stmts;
            full_body.extend(field_init_stmts);
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
            // Inject instance field initializers into default constructor
            for (f_name, f_init) in &instance_field_inits {
                let init_val = f_init.clone().unwrap_or(HirExpr::Lit(Literal::Undefined));
                body.push(HirStmt::Expr(HirExpr::MemberSet {
                    object: Box::new(HirExpr::Var(this_id)),
                    property: f_name.clone(),
                    value: Box::new(init_val),
                }));
            }
            
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
        for member in &class_decl.body.body {
            if let ClassElement::MethodDefinition(m) = member {
                if !m.r#static && m.kind != MethodDefinitionKind::Constructor {
                    let base_name = self.prop_name_to_string(&m.key);
                    let m_name = match m.kind {
                        MethodDefinitionKind::Get => format!("__get_{}", base_name),
                        MethodDefinitionKind::Set => format!("__set_{}", base_name),
                        MethodDefinitionKind::Method => base_name,
                        _ => base_name,
                    };
                    let m_func = methods.iter().find(|meth| meth.name == m_name).unwrap();
                    let func_name = format!("__bs_class_{}_{}", class_name, m_name);

                    self.function_stack.push(m_func.func_id);
                    self.push_scope();
                    let this_id = self.declare("this");
                    let old_this = self.this_binding;
                    self.this_binding = Some(this_id);

                    let mut params = vec![(this_id, "this".to_string())];
                    let mut param_destruct_stmts = Vec::new();
                    self.lower_formal_parameters(&m.value.params, &mut params, &mut param_destruct_stmts)?;

                    let body = match &m.value.body {
                        Some(b) => self.lower_function_body(b)?,
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
                        is_generator: m.value.generator,
                        is_async: m.value.r#async,
                    });
                }
            } else if let ClassElement::AccessorProperty(acc) = member {
                if !acc.r#static {
                    let base_name = self.prop_name_to_string(&acc.key);
                    let private_name = format!("__private_{}", base_name);
                    let get_name = format!("__get_{}", base_name);
                    let set_name = format!("__set_{}", base_name);

                    let get_func = methods.iter().find(|meth| meth.name == get_name).unwrap();
                    let set_func = methods.iter().find(|meth| meth.name == set_name).unwrap();

                    let get_func_name = format!("__bs_class_{}_{}", class_name, get_name);
                    self.push_scope();
                    let this_id = self.declare("this");
                    let get_body = vec![HirStmt::Return(Some(HirExpr::MemberGet {
                        object: Box::new(HirExpr::Var(this_id)),
                        property: private_name.clone(),
                    }))];
                    self.pop_scope();
                    self.functions.push(HirFunction {
                        id: get_func.func_id,
                        name: get_func_name,
                        params: vec![(this_id, "this".to_string())],
                        body: get_body,
                        captures: Vec::new(),
                        is_generator: false,
                        is_async: false,
                    });

                    let set_func_name = format!("__bs_class_{}_{}", class_name, set_name);
                    self.push_scope();
                    let this_id_set = self.declare("this");
                    let val_id = self.declare("v");
                    let set_body = vec![HirStmt::Expr(HirExpr::MemberSet {
                        object: Box::new(HirExpr::Var(this_id_set)),
                        property: private_name.clone(),
                        value: Box::new(HirExpr::Var(val_id)),
                    })];
                    self.pop_scope();
                    self.functions.push(HirFunction {
                        id: set_func.func_id,
                        name: set_func_name,
                        params: vec![(this_id_set, "this".to_string()), (val_id, "v".to_string())],
                        body: set_body,
                        captures: Vec::new(),
                        is_generator: false,
                        is_async: false,
                    });
                }
            }
        }

        // Lower static methods
        for member in &class_decl.body.body {
            if let ClassElement::MethodDefinition(m) = member {
                if m.r#static {
                    let base_name = self.prop_name_to_string(&m.key);
                    let m_name = match m.kind {
                        MethodDefinitionKind::Get => format!("__get_{}", base_name),
                        MethodDefinitionKind::Set => format!("__set_{}", base_name),
                        MethodDefinitionKind::Method => base_name,
                        _ => base_name,
                    };
                    let m_func = static_methods.iter().find(|meth| meth.name == m_name).unwrap();
                    let func_name = format!("__bs_class_{}_static_{}", class_name, m_name);

                    self.function_stack.push(m_func.func_id);
                    self.push_scope();

                    let mut params = Vec::new();
                    let mut param_destruct_stmts = Vec::new();
                    self.lower_formal_parameters(&m.value.params, &mut params, &mut param_destruct_stmts)?;

                    let body = match &m.value.body {
                        Some(b) => self.lower_function_body(b)?,
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
                        is_generator: m.value.generator,
                        is_async: m.value.r#async,
                    });
                }
            } else if let ClassElement::AccessorProperty(acc) = member {
                if acc.r#static {
                    let base_name = self.prop_name_to_string(&acc.key);
                    let private_name = format!("__private_{}", base_name);
                    let get_name = format!("__get_{}", base_name);
                    let set_name = format!("__set_{}", base_name);

                    let get_func = static_methods.iter().find(|meth| meth.name == get_name).unwrap();
                    let set_func = static_methods.iter().find(|meth| meth.name == set_name).unwrap();

                    let get_func_name = format!("__bs_class_{}_static_{}", class_name, get_name);
                    self.push_scope();
                    let this_id = self.declare("this");
                    let get_body = vec![HirStmt::Return(Some(HirExpr::MemberGet {
                        object: Box::new(HirExpr::Var(this_id)),
                        property: private_name.clone(),
                    }))];
                    self.pop_scope();
                    self.functions.push(HirFunction {
                        id: get_func.func_id,
                        name: get_func_name,
                        params: vec![(this_id, "this".to_string())],
                        body: get_body,
                        captures: Vec::new(),
                        is_generator: false,
                        is_async: false,
                    });

                    let set_func_name = format!("__bs_class_{}_static_{}", class_name, set_name);
                    self.push_scope();
                    let this_id_set = self.declare("this");
                    let val_id = self.declare("v");
                    let set_body = vec![HirStmt::Expr(HirExpr::MemberSet {
                        object: Box::new(HirExpr::Var(this_id_set)),
                        property: private_name.clone(),
                        value: Box::new(HirExpr::Var(val_id)),
                    })];
                    self.pop_scope();
                    self.functions.push(HirFunction {
                        id: set_func.func_id,
                        name: set_func_name,
                        params: vec![(this_id_set, "this".to_string()), (val_id, "v".to_string())],
                        body: set_body,
                        captures: Vec::new(),
                        is_generator: false,
                        is_async: false,
                    });
                }
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
            let mut stmts = Vec::new();
            for s in &sb.body {
                self.lower_stmt(s, &mut stmts)?;
            }
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
