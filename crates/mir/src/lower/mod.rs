//! HIR → MIR lowering.
//!
//! Flattens HIR expressions into three-address MIR instructions organized
//! into basic blocks with explicit control-flow edges.

use std::collections::HashMap;

use diagnostics::CompileResult;
use hir::{
    self, HirExpr, HirModule, HirStmt,
};
use crate::types::*;

mod stmt;
mod expr;
pub mod builtins;

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

    let mut global_fn_bindings = HashMap::new();
    for stmt in &hir.stmts {
        if let HirStmt::Let { binding, name, init: Some(HirExpr::Closure { .. }) } = stmt {
            global_fn_bindings.insert(name.clone(), *binding);
        }
    }

    let mut global_fn_captures = HashMap::new();
    for f in &hir.functions {
        global_fn_captures.insert(f.name.clone(), f.captures.clone());
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
            global_fn_bindings.clone(),
            global_fn_captures.clone(),
            false,
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
            global_fn_bindings.clone(),
            global_fn_captures.clone(),
            f.is_async && f.is_generator,
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
    global_fn_bindings: HashMap<String, hir::BindingId>,
    global_fn_captures: HashMap<String, Vec<hir::BindingId>>,
    is_async_generator: bool,
}

impl<'a> LowerCtx<'a> {
    fn new(
        func_names: HashMap<String, String>,
        classes: HashMap<String, hir::HirClass>,
        method_indices: HashMap<String, u32>,
        class_shapes: HashMap<String, u64>,
        capture_cells: &'a std::collections::HashSet<hir::BindingId>,
        global_fn_bindings: HashMap<String, hir::BindingId>,
        global_fn_captures: HashMap<String, Vec<hir::BindingId>>,
        is_async_generator: bool,
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
            global_fn_bindings,
            global_fn_captures,
            is_async_generator,
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
}
