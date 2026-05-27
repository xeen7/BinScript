//! HIR → MIR lowering.
//!
//! Flattens HIR expressions into three-address MIR instructions organized
//! into basic blocks with explicit control-flow edges.

use std::collections::HashMap;

use diagnostics::{CompileError, CompileResult};
use hir::{
    self, BinOp as HBinOp, HirExpr, HirModule, HirStmt, Literal,
    UnaryOp as HUnaryOp,
};

use crate::builtins::BuiltinFn;
use crate::types::*;

/// Lower an `HirModule` into an `MirModule`.
pub fn lower_module(hir: &HirModule) -> CompileResult<MirModule> {
    let mut classes = hir.classes.clone();
    classes.insert("CaptureCell".to_string(), hir::HirClass {
        name: "CaptureCell".to_string(),
        super_name: None,
        fields: vec!["value".to_string()],
        methods: Vec::new(),
        getters: Vec::new(),
        setters: Vec::new(),
        static_getters: Vec::new(),
        static_setters: Vec::new(),
    });

    // Register all user-defined function names first so calls can resolve.
    let mut func_names: HashMap<String, String> = HashMap::new();
    for f in &hir.functions {
        let mir_name = if f.name.starts_with("__bs_") {
            f.name.clone()
        } else {
            format!("__bs_{}", f.name)
        };
        func_names.insert(f.name.clone(), mir_name);
    }

    // Determine globally unique indices for all method names.
    let mut method_names: Vec<String> = classes.values()
        .flat_map(|c| c.methods.iter().map(|m| m.name.clone()))
        .collect();
    method_names.sort();
    method_names.dedup();
    let method_indices: HashMap<String, u32> = method_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, i as u32))
        .collect();

    // Assign sequential shape_ids starting at 1.
    let mut class_names: Vec<String> = classes.keys().cloned().collect();
    class_names.sort();
    let class_shapes: HashMap<String, u64> = class_names
        .into_iter()
        .enumerate()
        .map(|(i, name)| (name, (i + 1) as u64))
        .collect();

    // Lower top-level statements → main body.
    let main_body = {
        let mut ctx = LowerCtx::new(
            func_names.clone(),
            classes.clone(),
            method_indices.clone(),
            class_shapes.clone(),
            &hir.capture_cells,
        );
        ctx.lower_stmts(&hir.stmts)?;
        ctx.finish("__bs_main", false, false)
    };

    // Collect all top-level global function declaration IDs
    let mut global_func_ids = std::collections::HashSet::new();
    fn collect_global_funcs(stmts: &[HirStmt], ids: &mut std::collections::HashSet<hir::FuncId>) {
        for stmt in stmts {
            match stmt {
                HirStmt::FuncDecl { id, .. } => {
                    ids.insert(*id);
                }
                HirStmt::Block(inner) => {
                    collect_global_funcs(inner, ids);
                }
                _ => {}
            }
        }
    }
    collect_global_funcs(&hir.stmts, &mut global_func_ids);

    // Lower each function.
    let mut functions = Vec::new();
    for f in &hir.functions {
        let mut ctx = LowerCtx::new(
            func_names.clone(),
            classes.clone(),
            method_indices.clone(),
            class_shapes.clone(),
            &hir.capture_cells,
        );

        let mut params = Vec::new();
        
        // Determine if this function is a closure: either starts with __bs_closure_ or has captures,
        // or is a function/method that is not a class constructor/method.
        let is_closure = f.name.starts_with("__bs_closure_") 
            || !f.captures.is_empty() 
            || !f.name.starts_with("__bs_class_")
            || f.name.contains("_static_");
        
        let env_reg = if is_closure {
            let reg = ctx.fresh_reg();
            params.push((reg, "__env".to_string()));
            Some(reg)
        } else {
            None
        };

        // If it's a closure, load captures from the environment structure
        if let Some(env) = env_reg {
            for (i, &bid) in f.captures.iter().enumerate() {
                let loaded_reg = ctx.fresh_reg();
                // index 1 + i because index 0 is the function pointer
                ctx.emit(MirInstr::LoadField(loaded_reg, env, 1 + i as u32));
                ctx.bind(bid, loaded_reg);
                if ctx.capture_cells.contains(&bid) {
                    ctx.reg_shapes.insert(loaded_reg, "CaptureCell".to_string());
                }
            }
        }

        // Declare standard parameters
        for (bid, name) in &f.params {
            let reg = ctx.fresh_reg();
            ctx.bind(*bid, reg);
            if name == "this" && f.name.starts_with("__bs_class_") {
                let without_prefix = &f.name["__bs_class_".len()..];
                for class_name in classes.keys() {
                    if without_prefix == format!("{}_constructor", class_name)
                        || without_prefix.starts_with(&format!("{}_", class_name))
                    {
                        ctx.reg_shapes.insert(reg, class_name.clone());
                        break;
                    }
                }
            }
            params.push((reg, name.clone()));
        }

        ctx.lower_stmts(&f.body)?;
        // Implicit return undefined if needed.
        if !ctx.current_block_terminated() {
            ctx.emit(MirInstr::Return(Some(MirOperand::ConstUndefined)));
        }
        let mir_fn_name = if f.name.starts_with("__bs_") {
            f.name.clone()
        } else {
            format!("__bs_{}", f.name)
        };
        let mut mf = ctx.finish(&mir_fn_name, f.is_generator, f.is_async);
        mf.params = params;
        functions.push(mf);
    }

    let mut func_id_to_name = std::collections::HashMap::new();
    for f in &hir.functions {
        let mir_name = if f.name.starts_with("__bs_") {
            f.name.clone()
        } else {
            format!("__bs_{}", f.name)
        };
        func_id_to_name.insert(f.id, mir_name);
    }

    Ok(MirModule {
        functions,
        main_body,
        classes: classes.clone(),
        func_id_to_name,
    })
}

// ===========================================================================
// Lowering context
// ===========================================================================

struct LoopStackFrame {
    label: Option<String>,
    continue_target: Option<BlockId>,
    break_target: BlockId,
}

struct LowerCtx<'a> {
    next_reg: MirReg,
    next_block: BlockId,
    blocks: Vec<BasicBlock>,
    current: BlockId,
    bindings: HashMap<hir::BindingId, MirReg>,
    func_names: HashMap<String, String>,
    reg_shapes: HashMap<MirReg, String>,
    classes: HashMap<String, hir::HirClass>,
    method_indices: HashMap<String, u32>,
    class_shapes: HashMap<String, u64>,
    capture_cells: &'a std::collections::HashSet<hir::BindingId>,
    /// Map from MirReg to class name for registers holding class constructors.
    class_constructors: HashMap<MirReg, String>,
    /// Stack of loop/switch frames.
    loop_stack: Vec<LoopStackFrame>,
    next_loop_label: Option<String>,
    num_yield_points: u32,
    yield_saves: Vec<Vec<MirReg>>,
}

impl<'a> LowerCtx<'a> {
    fn new(
        func_names: HashMap<String, String>,
        classes: HashMap<String, hir::HirClass>,
        method_indices: HashMap<String, u32>,
        class_shapes: HashMap<String, u64>,
        capture_cells: &'a std::collections::HashSet<hir::BindingId>,
    ) -> Self {
        Self {
            next_reg: 0,
            next_block: 1,
            blocks: vec![BasicBlock { id: 0, instrs: Vec::new() }],
            current: 0,
            bindings: HashMap::new(),
            func_names,
            reg_shapes: HashMap::new(),
            classes,
            method_indices,
            class_shapes,
            capture_cells,
            class_constructors: HashMap::new(),
            loop_stack: Vec::new(),
            next_loop_label: None,
            num_yield_points: 0,
            yield_saves: Vec::new(),
        }
    }

    fn fresh_reg(&mut self) -> MirReg {
        let r = self.next_reg;
        self.next_reg += 1;
        r
    }

    fn has_getter(&self, class_name: &str, property: &str) -> bool {
        let mut curr = class_name;
        while let Some(class) = self.classes.get(curr) {
            if class.getters.iter().any(|g| g == property) {
                return true;
            }
            if let Some(ref super_name) = class.super_name {
                curr = super_name;
            } else {
                break;
            }
        }
        false
    }

    fn has_setter(&self, class_name: &str, property: &str) -> bool {
        let mut curr = class_name;
        while let Some(class) = self.classes.get(curr) {
            if class.setters.iter().any(|s| s == property) {
                return true;
            }
            if let Some(ref super_name) = class.super_name {
                curr = super_name;
            } else {
                break;
            }
        }
        false
    }

    fn has_static_getter(&self, class_name: &str, property: &str) -> bool {
        if let Some(class) = self.classes.get(class_name) {
            class.static_getters.iter().any(|g| g == property)
        } else {
            false
        }
    }

    fn has_static_setter(&self, class_name: &str, property: &str) -> bool {
        if let Some(class) = self.classes.get(class_name) {
            class.static_setters.iter().any(|s| s == property)
        } else {
            false
        }
    }

    fn fresh_block(&mut self) -> BlockId {
        let id = self.next_block;
        self.next_block += 1;
        self.blocks.push(BasicBlock { id, instrs: Vec::new() });
        id
    }

    fn bind(&mut self, hir_id: hir::BindingId, reg: MirReg) {
        self.bindings.insert(hir_id, reg);
    }

    fn emit(&mut self, instr: MirInstr) {
        let cur = self.current;
        if let Some(b) = self.blocks.iter_mut().find(|b| b.id == cur) {
            b.instrs.push(instr);
        }
    }

    fn switch_to(&mut self, id: BlockId) {
        self.current = id;
    }

    fn current_block_terminated(&self) -> bool {
        self.blocks
            .iter()
            .find(|b| b.id == self.current)
            .map(|b| {
                b.instrs.last().map_or(false, |i| {
                    matches!(
                        i,
                        MirInstr::Jump(_) | MirInstr::Branch(..) | MirInstr::Return(_) | MirInstr::Throw(_)
                    )
                })
            })
            .unwrap_or(false)
    }

    fn finish(self, name: &str, is_generator: bool, is_async: bool) -> MirFunction {
        MirFunction {
            name: name.to_string(),
            params: Vec::new(),
            blocks: self.blocks,
            next_reg: self.next_reg,
            next_block: self.next_block,
            is_generator,
            is_async,
            num_yield_points: self.num_yield_points,
            yield_saves: self.yield_saves,
        }
    }

    fn get_field_index(&self, class_name: &str, property: &str) -> Option<u32> {
        let fields = self.get_all_fields(class_name);
        fields.iter().position(|f| f == property).map(|idx| idx as u32)
    }

    fn get_all_fields(&self, class_name: &str) -> Vec<String> {
        let mut fields = Vec::new();
        if let Some(class) = self.classes.get(class_name) {
            if let Some(ref super_name) = class.super_name {
                fields.extend(self.get_all_fields(super_name));
            }
            fields.extend(class.fields.clone());
        }
        fields
    }

    // ── statements ─────────────────────────────────────────────────────────

    fn lower_stmts(&mut self, stmts: &[HirStmt]) -> CompileResult<()> {
        for s in stmts {
            self.lower_stmt(s)?;
            // Stop emitting after a terminator (throw, return, break, continue)
            if self.current_block_terminated() {
                break;
            }
        }
        Ok(())
    }

    fn lower_stmt(&mut self, stmt: &HirStmt) -> CompileResult<()> {
        match stmt {
            HirStmt::Expr(e) => {
                self.lower_expr(e)?;
                Ok(())
            }
            HirStmt::Let { binding, name, init } => {
                let reg = self.fresh_reg();
                self.bind(*binding, reg);
                
                if self.capture_cells.contains(binding) {
                    self.reg_shapes.insert(reg, "CaptureCell".to_string());
                    self.emit(MirInstr::Alloc(reg, "CaptureCell".to_string()));
                    let val = match init {
                        Some(e) => self.lower_expr(e)?,
                        None => MirOperand::ConstUndefined,
                    };
                    self.emit(MirInstr::StoreField(reg, 0, val.clone()));
                    // Track class constructor bindings for static getter/setter interception
                    if self.classes.contains_key(name) {
                        self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), val));
                    }
                } else {
                    let val = match init {
                        Some(e) => self.lower_expr(e)?,
                        None => MirOperand::ConstUndefined,
                    };
                    if let MirOperand::Reg(src_reg) = &val {
                        if let Some(shape) = self.reg_shapes.get(src_reg).cloned() {
                            self.reg_shapes.insert(reg, shape);
                        }
                    }
                    self.emit(MirInstr::Move(reg, val));
                    // Track class constructor bindings for static getter/setter interception
                    if self.classes.contains_key(name) {
                        self.class_constructors.insert(reg, name.clone());
                        self.emit(MirInstr::StoreGlobal(format!("__bs_class_val_{}", name), MirOperand::Reg(reg)));
                    }
                }
                Ok(())
            }
            HirStmt::Assign { target, value } => {
                let val = self.lower_expr(value)?;
                if let Some(&reg) = self.bindings.get(target) {
                    if self.capture_cells.contains(target) {
                        self.emit(MirInstr::StoreField(reg, 0, val));
                    } else {
                        if let MirOperand::Reg(src_reg) = &val {
                            if let Some(shape) = self.reg_shapes.get(src_reg).cloned() {
                                self.reg_shapes.insert(reg, shape);
                            }
                        }
                        self.emit(MirInstr::Move(reg, val));
                    }
                }
                Ok(())
            }
            HirStmt::If { cond, then_body, else_body } => {
                let cv = self.lower_expr(cond)?;
                let then_bb = self.fresh_block();
                let else_bb = self.fresh_block();
                let merge_bb = self.fresh_block();

                self.emit(MirInstr::Branch(cv, then_bb, else_bb));

                self.switch_to(then_bb);
                self.lower_stmts(then_body)?;
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(merge_bb));
                }

                self.switch_to(else_bb);
                if let Some(els) = else_body {
                    self.lower_stmts(els)?;
                }
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(merge_bb));
                }

                self.switch_to(merge_bb);
                Ok(())
            }
            HirStmt::While { cond, body } => {
                let cond_bb = self.fresh_block();
                let body_bb = self.fresh_block();
                let exit_bb = self.fresh_block();

                self.emit(MirInstr::Jump(cond_bb));

                self.switch_to(cond_bb);
                let cv = self.lower_expr(cond)?;
                self.emit(MirInstr::Branch(cv, body_bb, exit_bb));

                let label = self.next_loop_label.take();
                self.loop_stack.push(LoopStackFrame {
                    label,
                    continue_target: Some(cond_bb),
                    break_target: exit_bb,
                });
                self.switch_to(body_bb);
                self.lower_stmts(body)?;
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(cond_bb));
                }
                self.loop_stack.pop();

                self.switch_to(exit_bb);
                Ok(())
            }
            HirStmt::DoWhile { body, cond } => {
                let body_bb = self.fresh_block();
                let cond_bb = self.fresh_block();
                let exit_bb = self.fresh_block();

                self.emit(MirInstr::Jump(body_bb));

                let label = self.next_loop_label.take();
                self.loop_stack.push(LoopStackFrame {
                    label,
                    continue_target: Some(cond_bb),
                    break_target: exit_bb,
                });
                self.switch_to(body_bb);
                self.lower_stmts(body)?;
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(cond_bb));
                }
                self.loop_stack.pop();

                self.switch_to(cond_bb);
                let cv = self.lower_expr(cond)?;
                self.emit(MirInstr::Branch(cv, body_bb, exit_bb));

                self.switch_to(exit_bb);
                Ok(())
            }
            HirStmt::For { init, cond, update, body } => {
                if let Some(i) = init {
                    self.lower_stmt(i)?;
                }

                let cond_bb = self.fresh_block();
                let body_bb = self.fresh_block();
                let update_bb = self.fresh_block();
                let exit_bb = self.fresh_block();

                self.emit(MirInstr::Jump(cond_bb));

                self.switch_to(cond_bb);
                if let Some(c) = cond {
                    let cv = self.lower_expr(c)?;
                    self.emit(MirInstr::Branch(cv, body_bb, exit_bb));
                } else {
                    self.emit(MirInstr::Jump(body_bb));
                }

                let label = self.next_loop_label.take();
                self.loop_stack.push(LoopStackFrame {
                    label,
                    continue_target: Some(update_bb),
                    break_target: exit_bb,
                });
                self.switch_to(body_bb);
                self.lower_stmts(body)?;
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(update_bb));
                }
                self.loop_stack.pop();

                self.switch_to(update_bb);
                if let Some(u) = update {
                    self.lower_expr(u)?;
                }
                self.emit(MirInstr::Jump(cond_bb));

                self.switch_to(exit_bb);
                Ok(())
            }
            HirStmt::ForOf { left, right, body, is_await } => {
                let iter_reg = self.fresh_reg();
                let iter_val = self.lower_expr(right)?;
                self.emit(MirInstr::Move(iter_reg, iter_val));

                let cond_bb = self.fresh_block();
                let body_bb = self.fresh_block();
                let exit_bb = self.fresh_block();

                self.emit(MirInstr::Jump(cond_bb));
                self.switch_to(cond_bb);

                // Call generator.next()
                let next_val_reg = self.fresh_reg();
                self.emit(MirInstr::CallBuiltin(
                    next_val_reg,
                    BuiltinFn::GeneratorNext,
                    vec![MirOperand::Reg(iter_reg), MirOperand::ConstUndefined],
                ));

                let resolved_val_reg = if *is_await {
                    let yield_idx = self.num_yield_points;
                    self.num_yield_points += 1;
                    
                    let mut saves = Vec::new();
                    for r in 0..self.next_reg {
                        saves.push(r);
                    }
                    self.yield_saves.push(saves);

                    self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
                    let dest = self.fresh_reg();
                    self.emit(MirInstr::Resume(dest, yield_idx));
                    dest
                } else {
                    next_val_reg
                };

                // Check if iterator is done
                let is_done_reg = self.fresh_reg();
                self.emit(MirInstr::CallBuiltin(
                    is_done_reg,
                    BuiltinFn::GeneratorIsDone,
                    vec![MirOperand::Reg(iter_reg)],
                ));
                self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

                self.switch_to(body_bb);
                let label = self.next_loop_label.take();
                self.loop_stack.push(LoopStackFrame {
                    label,
                    continue_target: Some(cond_bb),
                    break_target: exit_bb,
                });
                
                // Declare the loop variable if it's a Let, or evaluate the assignee
                self.lower_stmt(left)?;
                
                // Assign to left
                match &**left {
                    HirStmt::Expr(HirExpr::Assign { target, .. }) => {
                        let reg = self.bindings[target];
                        if self.capture_cells.contains(target) {
                            self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                        } else {
                            self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                        }
                    }
                    HirStmt::Let { binding, name: _, init: _ } => {
                        let reg = self.bindings[binding];
                        if self.capture_cells.contains(binding) {
                            self.emit(MirInstr::StoreField(reg, 0, MirOperand::Reg(resolved_val_reg)));
                        } else {
                            self.emit(MirInstr::Move(reg, MirOperand::Reg(resolved_val_reg)));
                        }
                    }
                    _ => unreachable!("for..of left is neither assignment nor let"),
                }

                self.lower_stmts(body)?;
                
                if !self.current_block_terminated() {
                    self.emit(MirInstr::Jump(cond_bb));
                }
                
                self.loop_stack.pop();
                self.switch_to(exit_bb);

                Ok(())
            }
            HirStmt::Return(val) => {
                let v = match val {
                    Some(e) => Some(self.lower_expr(e)?),
                    None => None,
                };
                self.emit(MirInstr::Return(v));
                Ok(())
            }
            HirStmt::Break(label) => {
                if let Some(lbl) = label {
                    if let Some(frame) = self.loop_stack.iter().rev().find(|f| f.label.as_ref() == Some(lbl)) {
                        self.emit(MirInstr::Jump(frame.break_target));
                    }
                } else {
                    if let Some(frame) = self.loop_stack.last() {
                        self.emit(MirInstr::Jump(frame.break_target));
                    }
                }
                Ok(())
            }
            HirStmt::Continue(label) => {
                if let Some(lbl) = label {
                    if let Some(frame) = self.loop_stack.iter().rev().find(|f| f.label.as_ref() == Some(lbl)) {
                        if let Some(continue_target) = frame.continue_target {
                            self.emit(MirInstr::Jump(continue_target));
                        }
                    }
                } else {
                    if let Some(continue_target) = self.loop_stack.iter().rev().filter_map(|f| f.continue_target).next() {
                        self.emit(MirInstr::Jump(continue_target));
                    }
                }
                Ok(())
            }
            HirStmt::Switch { discriminant, cases } => {
                let disc_operand = self.lower_expr(discriminant)?;
                let exit_bb = self.fresh_block();
                
                let mut body_bbs = Vec::new();
                for _ in cases {
                    body_bbs.push(self.fresh_block());
                }

                self.loop_stack.push(LoopStackFrame {
                    label: None,
                    continue_target: None,
                    break_target: exit_bb,
                });

                let mut current_test_bb = self.current;
                let mut default_idx = None;

                for (i, case) in cases.iter().enumerate() {
                    if let Some(test_expr) = &case.test {
                        let next_test_bb = self.fresh_block();
                        self.switch_to(current_test_bb);

                        let test_operand = self.lower_expr(test_expr)?;
                        let eq_reg = self.fresh_reg();
                        self.emit(MirInstr::StrictEq(eq_reg, disc_operand.clone(), test_operand));
                        self.emit(MirInstr::Branch(MirOperand::Reg(eq_reg), body_bbs[i], next_test_bb));

                        current_test_bb = next_test_bb;
                    } else {
                        default_idx = Some(i);
                    }
                }

                self.switch_to(current_test_bb);
                if let Some(def_i) = default_idx {
                    self.emit(MirInstr::Jump(body_bbs[def_i]));
                } else {
                    self.emit(MirInstr::Jump(exit_bb));
                }

                for (i, case) in cases.iter().enumerate() {
                    self.switch_to(body_bbs[i]);
                    self.lower_stmts(&case.consequent)?;

                    if !self.current_block_terminated() {
                        let next_bb = if i + 1 < cases.len() {
                            body_bbs[i + 1]
                        } else {
                            exit_bb
                        };
                        self.emit(MirInstr::Jump(next_bb));
                    }
                }

                self.loop_stack.pop();
                self.switch_to(exit_bb);
                Ok(())
            }
            HirStmt::Block(stmts) => self.lower_stmts(stmts),
            HirStmt::Labeled { label, body } => {
                let old_label = self.next_loop_label.take();
                self.next_loop_label = Some(label.clone());
                
                let is_loop = matches!(
                    &**body,
                    HirStmt::While { .. }
                        | HirStmt::DoWhile { .. }
                        | HirStmt::For { .. }
                        | HirStmt::ForOf { .. }
                );

                if is_loop {
                    self.lower_stmt(body)?;
                } else {
                    let exit_bb = self.fresh_block();
                    self.loop_stack.push(LoopStackFrame {
                        label: Some(label.clone()),
                        continue_target: None,
                        break_target: exit_bb,
                    });
                    self.lower_stmt(body)?;
                    self.loop_stack.pop();
                    if !self.current_block_terminated() {
                        self.emit(MirInstr::Jump(exit_bb));
                    }
                    self.switch_to(exit_bb);
                }
                self.next_loop_label = old_label;
                Ok(())
            }
            HirStmt::FuncDecl { .. } => Ok(()), // handled at module level
            HirStmt::Throw(expr) => {
                let val = self.lower_expr(expr)?;
                self.emit(MirInstr::Throw(val));
                Ok(())
            }
            HirStmt::Try { body, catch_param, catch_body, finally_body } => {
                // 1. Allocate jmp_buf register
                let jmp_buf_reg = self.fresh_reg();
                self.emit(MirInstr::TryEnter(jmp_buf_reg));

                // 2. Call setjmp — returns 0 on first call, non-zero on longjmp
                let setjmp_result = self.fresh_reg();
                self.emit(MirInstr::SetJmp(setjmp_result, jmp_buf_reg));

                // 3. Branch: if setjmp returned non-zero → catch, else → try body
                let try_body_bb = self.fresh_block();
                let catch_bb = self.fresh_block();
                let finally_bb = self.fresh_block();
                let merge_bb = self.fresh_block();

                self.emit(MirInstr::Branch(MirOperand::Reg(setjmp_result), catch_bb, try_body_bb));

                // 4. Try body
                self.switch_to(try_body_bb);
                self.lower_stmts(body)?;
                if !self.current_block_terminated() {
                    self.emit(MirInstr::TryExit);
                    if finally_body.is_some() {
                        self.emit(MirInstr::Jump(finally_bb));
                    } else {
                        self.emit(MirInstr::Jump(merge_bb));
                    }
                }

                // 5. Catch body
                self.switch_to(catch_bb);
                // Get the thrown exception and bind it
                if let Some((bid, _name)) = catch_param {
                    let exc_reg = self.fresh_reg();
                    self.bind(*bid, exc_reg);
                    self.emit(MirInstr::CallDirect(
                        exc_reg,
                        "__bs_get_and_clear_exception".to_string(),
                        vec![],
                    ));
                } else {
                    // No catch param — still need to clear exception
                    let unused = self.fresh_reg();
                    self.emit(MirInstr::CallDirect(
                        unused,
                        "__bs_get_and_clear_exception".to_string(),
                        vec![],
                    ));
                }
                self.lower_stmts(catch_body)?;
                if !self.current_block_terminated() {
                    if finally_body.is_some() {
                        self.emit(MirInstr::Jump(finally_bb));
                    } else {
                        self.emit(MirInstr::Jump(merge_bb));
                    }
                }

                // 6. Finally body (if present)
                if let Some(fin_stmts) = finally_body {
                    self.switch_to(finally_bb);
                    self.lower_stmts(fin_stmts)?;
                    if !self.current_block_terminated() {
                        self.emit(MirInstr::Jump(merge_bb));
                    }
                }

                self.switch_to(merge_bb);
                Ok(())
            }
        }
    }

    // ── expressions ────────────────────────────────────────────────────────

    fn lower_expr(&mut self, expr: &HirExpr) -> CompileResult<MirOperand> {
        match expr {
            HirExpr::Lit(lit) => Ok(match lit {
                Literal::Number(n) => MirOperand::ConstNum(*n),
                Literal::String(s) => MirOperand::ConstStr(s.clone()),
                Literal::Bool(b) => MirOperand::ConstBool(*b),
                Literal::Null => MirOperand::ConstNull,
                Literal::Undefined => MirOperand::ConstUndefined,
            }),
            HirExpr::Var(bid) => {
                if let Some(&reg) = self.bindings.get(bid) {
                    if self.capture_cells.contains(bid) {
                        let dest = self.fresh_reg();
                        self.emit(MirInstr::LoadField(dest, reg, 0));
                        Ok(MirOperand::Reg(dest))
                    } else {
                        Ok(MirOperand::Reg(reg))
                    }
                } else {
                    Err(CompileError::Lowering {
                        message: format!("Unresolved binding {}", bid),
                    })
                }
            }
            HirExpr::BinOp(op, left, right) => {
                if matches!(op, HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing) {
                    let l = self.lower_expr(left)?;
                    let dest = self.fresh_reg();
                    let eval_r_bb = self.fresh_block();
                    let merge_bb = self.fresh_block();

                    self.emit(MirInstr::Move(dest, l.clone()));
                    if let HBinOp::And = op {
                        // AND: if left is true, evaluate right; else keep left (falsy)
                        self.emit(MirInstr::Branch(l, eval_r_bb, merge_bb));
                    } else if let HBinOp::Or = op {
                        // OR: if left is true, keep left (truthy); else evaluate right
                        self.emit(MirInstr::Branch(l, merge_bb, eval_r_bb));
                    } else {
                        // Nullish Coalescing (??): if left is null/undefined, evaluate right; else keep left
                        let cond_reg = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(cond_reg, "__bs_is_nullish".to_string(), vec![l]));
                        self.emit(MirInstr::Branch(MirOperand::Reg(cond_reg), eval_r_bb, merge_bb));
                    }

                    self.switch_to(eval_r_bb);
                    let r = self.lower_expr(right)?;
                    self.emit(MirInstr::Move(dest, r));
                    self.emit(MirInstr::Jump(merge_bb));

                    self.switch_to(merge_bb);
                    return Ok(MirOperand::Reg(dest));
                }

                let l = self.lower_expr(left)?;
                let r = self.lower_expr(right)?;
                let dest = self.fresh_reg();
                let instr = match op {
                    HBinOp::Add => MirInstr::Add(dest, l, r),
                    HBinOp::Sub => MirInstr::Sub(dest, l, r),
                    HBinOp::Mul => MirInstr::Mul(dest, l, r),
                    HBinOp::Div => MirInstr::Div(dest, l, r),
                    HBinOp::Mod => MirInstr::Mod(dest, l, r),
                    HBinOp::Exp => MirInstr::Exp(dest, l, r),
                    HBinOp::Eq => MirInstr::Eq(dest, l, r),
                    HBinOp::Ne => MirInstr::Ne(dest, l, r),
                    HBinOp::StrictEq => MirInstr::StrictEq(dest, l, r),
                    HBinOp::StrictNe => MirInstr::StrictNe(dest, l, r),
                    HBinOp::Lt => MirInstr::Lt(dest, l, r),
                    HBinOp::Le => MirInstr::Le(dest, l, r),
                    HBinOp::Gt => MirInstr::Gt(dest, l, r),
                    HBinOp::Ge => MirInstr::Ge(dest, l, r),
                    HBinOp::In => MirInstr::In(dest, l, r),
                    HBinOp::And | HBinOp::Or | HBinOp::NullishCoalescing => unreachable!(),
                    _ => MirInstr::Add(dest, l, r),
                };
                self.emit(instr);
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::UnaryOp(op, arg) => {
                let v = self.lower_expr(arg)?;
                let dest = self.fresh_reg();
                match op {
                    HUnaryOp::Plus => self.emit(MirInstr::Plus(dest, v)),
                    HUnaryOp::Neg => self.emit(MirInstr::Neg(dest, v)),
                    HUnaryOp::Not => self.emit(MirInstr::Not(dest, v)),
                    HUnaryOp::Typeof => self.emit(MirInstr::CallDirect(dest, "__bs_typeof".to_string(), vec![v])),
                    _ => self.emit(MirInstr::Move(dest, v)),
                }
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::Call { callee, args } => {
                let mir_args: Vec<MirOperand> = args
                    .iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<CompileResult<_>>()?;
                let dest = self.fresh_reg();
                match &**callee {
                    HirExpr::GlobalRef(name) => {
                        let fn_name = if name == "parseInt" {
                            if mir_args.len() == 1 {
                                "__bs_parseInt_1".to_string()
                            } else {
                                "__bs_parseInt_2".to_string()
                            }
                        } else {
                            self
                                .func_names
                                .get(name)
                                .cloned()
                                .unwrap_or_else(|| {
                                    if name.starts_with("__bs_") {
                                        name.clone()
                                    } else {
                                        format!("__bs_{}", name)
                                    }
                                })
                        };
                        self.emit(MirInstr::CallDirect(dest, fn_name, mir_args));
                    }
                    _ => {
                        let callee_op = self.lower_expr(callee)?;
                        let callee_reg = match callee_op {
                            MirOperand::Reg(r) => r,
                            other => {
                                let r = self.fresh_reg();
                                self.emit(MirInstr::Move(r, other));
                                r
                            }
                        };
                        let mut call_args = vec![MirOperand::Reg(callee_reg)];
                        call_args.extend(mir_args);
                        self.emit(MirInstr::CallClosure(dest, callee_reg, call_args));
                    }
                }
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::MemberCall { object, method, args } => {
                let mir_args: Vec<MirOperand> = args
                    .iter()
                    .map(|a| self.lower_expr(a))
                    .collect::<CompileResult<_>>()?;
                let dest = self.fresh_reg();
                if object == "console" && method == "log" {
                    self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::ConsoleLog, mir_args));
                } else if object == "Promise" && method == "all_2" {
                    self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseAll2, mir_args));
                } else if object == "Promise" && method == "race_2" {
                    self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::PromiseRace2, mir_args));
                } else if object == "Number" && method == "isInteger" {
                    self.emit(MirInstr::CallDirect(dest, "__bs_number_isInteger".to_string(), mir_args));
                } else if object == "Number" && method == "isFinite" {
                    self.emit(MirInstr::CallDirect(dest, "__bs_isFinite".to_string(), mir_args));
                } else if object == "Number" && method == "isNaN" {
                    self.emit(MirInstr::CallDirect(dest, "__bs_isNaN".to_string(), mir_args));
                } else if object == "Object" {
                    let is_obj_static = matches!(
                        method.as_str(),
                        "keys" | "values" | "entries" | "assign" | "create" | "getPrototypeOf" | "fromEntries"
                    );
                    if is_obj_static {
                        self.emit(MirInstr::CallDirect(dest, format!("__bs_object_{}", method), mir_args));
                    } else {
                        return Err(CompileError::Lowering {
                            message: format!("Object.{}() not supported", method),
                        });
                    }
                } else if object == "String" {
                    let is_str_static = matches!(
                        method.as_str(),
                        "fromCharCode" | "fromCodePoint"
                    );
                    if is_str_static {
                        self.emit(MirInstr::CallDirect(dest, format!("__bs_string_{}", method), mir_args));
                    } else {
                        return Err(CompileError::Lowering {
                            message: format!("String.{}() not supported", method),
                        });
                    }
                } else if object == "Date" && method == "now" {
                    self.emit(MirInstr::CallDirect(dest, "__bs_date_now".to_string(), mir_args));
                } else if object == "Math" {
                    let is_math = matches!(
                        method.as_str(),
                        "floor" | "ceil" | "round" | "abs" | "sqrt" | "pow" |
                        "min" | "max" | "log" | "log2" | "sin" | "cos" | "tan" |
                        "random" | "trunc"
                    );
                    if is_math {
                        self.emit(MirInstr::CallDirect(dest, format!("__bs_math_{}", method), mir_args));
                    } else {
                        return Err(CompileError::Lowering {
                            message: format!("Math.{}() not supported", method),
                        });
                    }
                } else {
                    return Err(CompileError::Lowering {
                        message: format!("{}.{}() not supported", object, method),
                    });
                }
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::Assign { target, value } => {
                let val = self.lower_expr(value)?;
                // Look up the binding; if it's unknown (e.g. a temp from object literal lowering),
                // auto-allocate a fresh register for it so it can be used later.
                let reg = if let Some(&existing) = self.bindings.get(target) {
                    existing
                } else {
                    let fresh = self.fresh_reg();
                    self.bindings.insert(*target, fresh);
                    fresh
                };
                if self.capture_cells.contains(target) {
                    self.emit(MirInstr::StoreField(reg, 0, val.clone()));
                } else {
                    if let MirOperand::Reg(src_reg) = &val {
                        if let Some(shape) = self.reg_shapes.get(src_reg).cloned() {
                            self.reg_shapes.insert(reg, shape);
                        }
                    }
                    self.emit(MirInstr::Move(reg, val.clone()));
                }
                Ok(val)
            }
            HirExpr::Ternary { cond, then_expr, else_expr } => {
                let cv = self.lower_expr(cond)?;
                let dest = self.fresh_reg();
                let then_bb = self.fresh_block();
                let else_bb = self.fresh_block();
                let merge_bb = self.fresh_block();

                self.emit(MirInstr::Branch(cv, then_bb, else_bb));

                self.switch_to(then_bb);
                let tv = self.lower_expr(then_expr)?;
                self.emit(MirInstr::Move(dest, tv));
                self.emit(MirInstr::Jump(merge_bb));

                self.switch_to(else_bb);
                let ev = self.lower_expr(else_expr)?;
                self.emit(MirInstr::Move(dest, ev));
                self.emit(MirInstr::Jump(merge_bb));

                self.switch_to(merge_bb);
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::DeleteProp { object, property } => {
                let obj = self.lower_expr(object)?;
                let prop = self.lower_expr(property)?;
                let dest = self.fresh_reg();
                self.emit(MirInstr::DeleteProp(dest, obj, prop));
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::Yield { arg, delegate } => {
                if *delegate {
                    let iter_val = match arg {
                        Some(e) => self.lower_expr(e)?,
                        None => MirOperand::ConstUndefined,
                    };
                    let iter_reg = self.fresh_reg();
                    self.emit(MirInstr::Move(iter_reg, iter_val));

                    let sent_reg = self.fresh_reg();
                    self.emit(MirInstr::Move(sent_reg, MirOperand::ConstUndefined));

                    let result_reg = self.fresh_reg();

                    let cond_bb = self.fresh_block();
                    let body_bb = self.fresh_block();
                    let exit_bb = self.fresh_block();

                    self.emit(MirInstr::Jump(cond_bb));
                    self.switch_to(cond_bb);

                    let next_val_reg = self.fresh_reg();
                    self.emit(MirInstr::CallBuiltin(
                        next_val_reg,
                        BuiltinFn::GeneratorNext,
                        vec![MirOperand::Reg(iter_reg), MirOperand::Reg(sent_reg)],
                    ));

                    let is_done_reg = self.fresh_reg();
                    self.emit(MirInstr::CallBuiltin(
                        is_done_reg,
                        BuiltinFn::GeneratorIsDone,
                        vec![MirOperand::Reg(iter_reg)],
                    ));
                    self.emit(MirInstr::Branch(MirOperand::Reg(is_done_reg), exit_bb, body_bb));

                    // Body
                    self.switch_to(body_bb);
                    
                    let yield_idx = self.num_yield_points;
                    self.num_yield_points += 1;
                    let mut saves = Vec::new();
                    for r in 0..self.next_reg {
                        saves.push(r);
                    }
                    self.yield_saves.push(saves);

                    self.emit(MirInstr::Suspend(yield_idx, MirOperand::Reg(next_val_reg)));
                    self.emit(MirInstr::Resume(sent_reg, yield_idx));
                    self.emit(MirInstr::Jump(cond_bb));

                    // Exit
                    self.switch_to(exit_bb);
                    self.emit(MirInstr::Move(result_reg, MirOperand::Reg(next_val_reg)));

                    Ok(MirOperand::Reg(result_reg))
                } else {
                    let v = match arg {
                        Some(e) => self.lower_expr(e)?,
                        None => MirOperand::ConstUndefined,
                    };
                    let yield_idx = self.num_yield_points;
                    self.num_yield_points += 1;
                    
                    // Save all registers assigned up to this point
                    let mut saves = Vec::new();
                    for r in 0..self.next_reg {
                        saves.push(r);
                    }
                    self.yield_saves.push(saves);

                    self.emit(MirInstr::Suspend(yield_idx, v));
                    let dest = self.fresh_reg();
                    self.emit(MirInstr::Resume(dest, yield_idx));
                    Ok(MirOperand::Reg(dest))
                }
            }
            HirExpr::Await(inner) => {
                let v = self.lower_expr(inner)?;
                let yield_idx = self.num_yield_points;
                self.num_yield_points += 1;
                
                let mut saves = Vec::new();
                for r in 0..self.next_reg {
                    saves.push(r);
                }
                self.yield_saves.push(saves);

                self.emit(MirInstr::Suspend(yield_idx, v));
                let dest = self.fresh_reg();
                self.emit(MirInstr::Resume(dest, yield_idx));
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::Seq(exprs) => {
                let mut last = MirOperand::ConstUndefined;
                for e in exprs {
                    last = self.lower_expr(e)?;
                }
                Ok(last)
            }
            HirExpr::GlobalRef(name) => {
                if name.starts_with("__bs_class_val_") {
                    let dest = self.fresh_reg();
                    let class_name = name["__bs_class_val_".len()..].to_string();
                    self.emit(MirInstr::LoadGlobal(dest, name.clone()));
                    // Track as class constructor for static getter/setter resolution
                    if self.classes.contains_key(&class_name) {
                        self.class_constructors.insert(dest, class_name);
                    }
                    return Ok(MirOperand::Reg(dest));
                }
                match name.as_str() {
                    "NaN" => Ok(MirOperand::ConstNum(std::f64::NAN)),
                    "Infinity" => Ok(MirOperand::ConstNum(std::f64::INFINITY)),
                    "undefined" => Ok(MirOperand::ConstUndefined),
                    "globalThis" => {
                        let dest = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(dest, "__bs_get_globalThis".to_string(), vec![]));
                        Ok(MirOperand::Reg(dest))
                    }
                    "Symbol" => {
                        let dest = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(dest, "__bs_get_Symbol_global".to_string(), vec![]));
                        Ok(MirOperand::Reg(dest))
                    }
                    _ => Ok(MirOperand::ConstUndefined),
                }
            }
            
            // --- Stage 6 additions ---
            HirExpr::JsonTape(bytes) => {
                let s = String::from_utf8_lossy(bytes).into_owned();
                let dest = self.fresh_reg();
                
                // Emitting it as a builtin/intrinsic call
                // For string literals we can pass them as a single string argument, 
                // but our intrinsic takes (ptr, len).
                // Let's create a new builtin for JsonParseLazy that codegen handles.
                self.emit(MirInstr::CallBuiltin(dest, BuiltinFn::JsonParseLazy, vec![MirOperand::ConstStr(s)]));
                Ok(MirOperand::Reg(dest))
            }

            // --- Stage 2 additions ---
            HirExpr::New { class_name, args } => {
                let mut mir_args = Vec::new();
                for a in args {
                    mir_args.push(self.lower_expr(a)?);
                }
                let obj_reg = self.fresh_reg();
                self.reg_shapes.insert(obj_reg, class_name.clone());

                self.emit(MirInstr::Alloc(obj_reg, class_name.clone()));

                let mut ctor_args = vec![MirOperand::Reg(obj_reg)];
                ctor_args.extend(mir_args);

                let unused = self.fresh_reg();
                let ctor_name = format!("__bs_class_{}_constructor", class_name);
                self.emit(MirInstr::CallDirect(unused, ctor_name, ctor_args));

                Ok(MirOperand::Reg(obj_reg))
            }
            HirExpr::MemberGet { object, property } => {
                let obj_operand = self.lower_expr(object)?;
                let obj_reg = match obj_operand {
                    MirOperand::Reg(r) => r,
                    _ => {
                        let r = self.fresh_reg();
                        self.emit(MirInstr::Move(r, obj_operand));
                        r
                    }
                };
                
                let dest = self.fresh_reg();
                if let Some(shape) = self.reg_shapes.get(&obj_reg) {
                    if self.has_getter(shape, property) {
                        let getter_name = format!("__get_{}", property);
                        if let Some(&method_idx) = self.method_indices.get(&getter_name) {
                            let mir_args = vec![MirOperand::Reg(obj_reg)];
                            self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                            return Ok(MirOperand::Reg(dest));
                        }
                    }
                    if let Some(index) = self.get_field_index(shape, property) {
                        self.emit(MirInstr::LoadField(dest, obj_reg, index));
                        return Ok(MirOperand::Reg(dest));
                    }
                }
                // Check for static getter on class constructor
                if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
                    if self.has_static_getter(&ctor_class, property) {
                        let getter_prop = format!("__get_{}", property);
                        let closure_reg = self.fresh_reg();
                        self.emit(MirInstr::LoadProp(closure_reg, obj_reg, getter_prop));
                        self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg)]));
                        return Ok(MirOperand::Reg(dest));
                    }
                }
                
                // Fallback to dynamic property get (for JsonTape and untyped objects)
                self.emit(MirInstr::LoadProp(dest, obj_reg, property.clone()));
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::MemberSet { object, property, value } => {
                let obj_operand = self.lower_expr(object)?;
                let obj_reg = match obj_operand {
                    MirOperand::Reg(r) => r,
                    _ => {
                        let r = self.fresh_reg();
                        self.emit(MirInstr::Move(r, obj_operand));
                        r
                    }
                };
                
                let val = self.lower_expr(value)?;

                if let Some(shape) = self.reg_shapes.get(&obj_reg) {
                    if self.has_setter(shape, property) {
                        let setter_name = format!("__set_{}", property);
                        if let Some(&method_idx) = self.method_indices.get(&setter_name) {
                            let dest = self.fresh_reg();
                            let mir_args = vec![MirOperand::Reg(obj_reg), val.clone()];
                            self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                            return Ok(val);
                        }
                    }
                    if let Some(index) = self.get_field_index(shape, property) {
                        self.emit(MirInstr::StoreField(obj_reg, index, val.clone()));
                        return Ok(val);
                    }
                }
                
                // Check for static setter on class constructor
                if let Some(ctor_class) = self.class_constructors.get(&obj_reg).cloned() {
                    if self.has_static_setter(&ctor_class, property) {
                        let setter_prop = format!("__set_{}", property);
                        let closure_reg = self.fresh_reg();
                        self.emit(MirInstr::LoadProp(closure_reg, obj_reg, setter_prop));
                        let dest = self.fresh_reg();
                        self.emit(MirInstr::CallClosure(dest, closure_reg, vec![MirOperand::Reg(closure_reg), val.clone()]));
                        return Ok(val);
                    }
                }
                
                // Fallback to dynamic property set
                self.emit(MirInstr::StoreProp(obj_reg, property.clone(), val.clone()));
                Ok(val)
            }
            HirExpr::MethodCall { object, method, args } => {
                let obj_operand = self.lower_expr(object)?;
                
                let is_builtin = matches!(
                    method.as_str(),
                    "push" | "pop" | "slice" | "indexOf" | "includes" | "join" | "reverse" |
                    "concat" | "fill" | "forEach" | "map" | "filter" | "find" | "findIndex" |
                    "every" | "some" | "reduce" | "charAt" | "charCodeAt" | "startsWith" |
                    "endsWith" | "substring" | "split" | "trim" | "toUpperCase" | "toLowerCase" |
                    "replace" | "repeat" | "padStart" | "padEnd" | "getTime" | "getFullYear" |
                    "getMonth" | "getDate" | "getHours" | "getMinutes" | "getSeconds" | "toString" | "valueOf"
                );

                if is_builtin {
                    let method_idx = self.method_indices.get(method).map(|&i| i as f64).unwrap_or(-1.0);
                    let mut mir_args = vec![obj_operand];
                    for a in args {
                        mir_args.push(self.lower_expr(a)?);
                    }
                    mir_args.push(MirOperand::ConstNum(method_idx));

                    let dest = self.fresh_reg();
                    self.emit(MirInstr::CallDirect(
                        dest,
                        format!("__bs_call_{}", method),
                        mir_args,
                    ));
                    Ok(MirOperand::Reg(dest))
                } else {
                    let obj_reg = match obj_operand {
                        MirOperand::Reg(r) => r,
                        _ => {
                            let r = self.fresh_reg();
                            self.emit(MirInstr::Move(r, obj_operand));
                            r
                        }
                    };

                    let has_spread = args.iter().any(|a| matches!(a, HirExpr::Spread(_)));
                    if has_spread {
                        let args_array = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(args_array, "__bs_array_new".to_string(), vec![]));
                        
                        // Prepend the receiver obj
                        let unused = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(
                            unused,
                            "__bs_array_push".to_string(),
                            vec![MirOperand::Reg(args_array), MirOperand::Reg(obj_reg)],
                        ));
                        
                        for a in args {
                            if let HirExpr::Spread(inner) = a {
                                let op = self.lower_expr(inner)?;
                                let unused = self.fresh_reg();
                                self.emit(MirInstr::CallDirect(
                                    unused,
                                    "__bs_array_push_spread".to_string(),
                                    vec![MirOperand::Reg(args_array), op],
                                ));
                            } else {
                                let op = self.lower_expr(a)?;
                                let unused = self.fresh_reg();
                                self.emit(MirInstr::CallDirect(
                                    unused,
                                    "__bs_array_push".to_string(),
                                    vec![MirOperand::Reg(args_array), op],
                                ));
                            }
                        }
                        
                        let dest = self.fresh_reg();
                        if let Some(&method_idx) = self.method_indices.get(method) {
                            self.emit(MirInstr::CallDirect(
                                dest,
                                "__bs_vcall_apply".to_string(),
                                vec![
                                    MirOperand::Reg(obj_reg),
                                    MirOperand::ConstNum(method_idx as f64),
                                    MirOperand::Reg(args_array),
                                ],
                            ));
                        } else {
                            let fn_reg = self.fresh_reg();
                            self.emit(MirInstr::LoadProp(fn_reg, obj_reg, method.clone()));
                            self.emit(MirInstr::CallDirect(
                                dest,
                                "__bs_call_apply".to_string(),
                                vec![
                                    MirOperand::Reg(fn_reg),
                                    MirOperand::ConstUndefined,
                                    MirOperand::Reg(args_array),
                                ],
                            ));
                        }
                        return Ok(MirOperand::Reg(dest));
                    }
                    
                    if let Some(&method_idx) = self.method_indices.get(method) {
                        let mut mir_args = vec![MirOperand::Reg(obj_reg)];
                        for a in args {
                            mir_args.push(self.lower_expr(a)?);
                        }

                        let dest = self.fresh_reg();
                        self.emit(MirInstr::CallVTable(dest, obj_reg, method_idx, mir_args));
                        Ok(MirOperand::Reg(dest))
                    } else {
                        // Fallback: load method as a closure from dynamic property and call it
                        let fn_reg = self.fresh_reg();
                        self.emit(MirInstr::LoadProp(fn_reg, obj_reg, method.clone()));
                        let mut call_args = vec![MirOperand::Reg(fn_reg)];
                        for a in args {
                            call_args.push(self.lower_expr(a)?);
                        }
                        let dest = self.fresh_reg();
                        self.emit(MirInstr::CallClosure(dest, fn_reg, call_args));
                        Ok(MirOperand::Reg(dest))
                    }
                }
            }
            HirExpr::InstanceOf { expr, class_name } => {
                let ev = self.lower_expr(expr)?;
                let shape_id = *self.class_shapes.get(class_name).ok_or_else(|| {
                    CompileError::Lowering {
                        message: format!("Class '{}' in instanceof check is not defined", class_name),
                    }
                })?;
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_instanceof".to_string(),
                    vec![ev, MirOperand::ConstNum(shape_id as f64)],
                ));
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::Closure { func_id, captures } => {
                let mut mir_caps = Vec::new();
                for &bid in captures {
                    if let Some(&reg) = self.bindings.get(&bid) {
                        mir_caps.push(MirOperand::Reg(reg));
                    } else {
                        return Err(CompileError::Lowering {
                            message: format!("Captured binding {} not resolved in bindings", bid),
                        });
                    }
                }
                let dest = self.fresh_reg();
                self.emit(MirInstr::AllocClosure(dest, *func_id, mir_caps));
                Ok(MirOperand::Reg(dest))
            }
            // --- Stage 11 additions ---
            HirExpr::ArrayLit(elems) => {
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(dest, "__bs_array_new".to_string(), vec![]));
                for elem in elems {
                    if let HirExpr::Spread(inner) = elem {
                        let operand = self.lower_expr(inner)?;
                        let unused = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(
                            unused,
                            "__bs_array_push_spread".to_string(),
                            vec![MirOperand::Reg(dest), operand],
                        ));
                    } else {
                        let operand = self.lower_expr(elem)?;
                        let unused = self.fresh_reg();
                        self.emit(MirInstr::CallDirect(
                            unused,
                            "__bs_array_push".to_string(),
                            vec![MirOperand::Reg(dest), operand],
                        ));
                    }
                }
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::IndexGet { object, index } => {
                let obj_operand = self.lower_expr(object)?;
                let idx_operand = self.lower_expr(index)?;
                let dest = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    dest,
                    "__bs_index_get".to_string(),
                    vec![obj_operand, idx_operand],
                ));
                Ok(MirOperand::Reg(dest))
            }
            HirExpr::IndexSet { object, index, value } => {
                let obj_operand = self.lower_expr(object)?;
                let idx_operand = self.lower_expr(index)?;
                let val_operand = self.lower_expr(value)?;
                let unused = self.fresh_reg();
                self.emit(MirInstr::CallDirect(
                    unused,
                    "__bs_index_set".to_string(),
                    vec![obj_operand, idx_operand, val_operand.clone()],
                ));
                Ok(val_operand)
            }
            HirExpr::Spread(_) => {
                Err(CompileError::Lowering {
                    message: "Spread expression outside array/object literals is unsupported".into(),
                })
            }
        }
    }
}
