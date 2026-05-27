//! MIR type definitions — three-address, basic-block IR.

use crate::builtins::BuiltinFn;

/// Virtual register identifier.
pub type MirReg = u32;

/// Basic-block identifier.
pub type BlockId = u32;

// ---------------------------------------------------------------------------
// Operands
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum MirOperand {
    Reg(MirReg),
    ConstNum(f64),
    ConstBool(bool),
    ConstStr(String),
    ConstNull,
    ConstUndefined,
}

// ---------------------------------------------------------------------------
// Instructions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub enum MirInstr {
    // ── arithmetic ─────────────────────────────────────────────────────────
    Add(MirReg, MirOperand, MirOperand),
    Sub(MirReg, MirOperand, MirOperand),
    Mul(MirReg, MirOperand, MirOperand),
    Div(MirReg, MirOperand, MirOperand),
    Mod(MirReg, MirOperand, MirOperand),
    Exp(MirReg, MirOperand, MirOperand),
    Neg(MirReg, MirOperand),
    Plus(MirReg, MirOperand),

    // ── comparison ─────────────────────────────────────────────────────────
    Eq(MirReg, MirOperand, MirOperand),
    Ne(MirReg, MirOperand, MirOperand),
    StrictEq(MirReg, MirOperand, MirOperand),
    StrictNe(MirReg, MirOperand, MirOperand),
    Lt(MirReg, MirOperand, MirOperand),
    Le(MirReg, MirOperand, MirOperand),
    Gt(MirReg, MirOperand, MirOperand),
    Ge(MirReg, MirOperand, MirOperand),
    In(MirReg, MirOperand, MirOperand),

    // ── logical ────────────────────────────────────────────────────────────
    Not(MirReg, MirOperand),

    // ── data movement ──────────────────────────────────────────────────────
    Move(MirReg, MirOperand),

    // ── calls ──────────────────────────────────────────────────────────────
    CallDirect(MirReg, String, Vec<MirOperand>),
    CallBuiltin(MirReg, BuiltinFn, Vec<MirOperand>),

    // ── control flow ───────────────────────────────────────────────────────
    Branch(MirOperand, BlockId, BlockId),
    Jump(BlockId),
    Return(Option<MirOperand>),

    // --- Stage 2 additions ---
    /// Allocate an object with a given shape: `Alloc(dest, class_name)`
    Alloc(MirReg, String),
    /// Load a field from an object at a static index: `LoadField(dest, obj_reg, index)`
    LoadField(MirReg, MirReg, u32),
    /// Store a value to a field of an object at a static index: `StoreField(obj_reg, index, val_operand)`
    StoreField(MirReg, u32, MirOperand),
    /// Call a method via vtable: `CallVTable(dest, obj_reg, method_index, args)`
    CallVTable(MirReg, MirReg, u32, Vec<MirOperand>),
    /// Dynamic property read: `LoadProp(dest, obj_reg, property_name)`
    LoadProp(MirReg, MirReg, String),
    /// Dynamic property write: `StoreProp(obj_reg, property_name, val_operand)`
    StoreProp(MirReg, String, MirOperand),
    /// Dynamic property delete: `DeleteProp(dest, obj_reg, prop_operand)`
    DeleteProp(MirReg, MirOperand, MirOperand),
    // --- Stage 3 additions ---
    /// Allocate a closure object: `AllocClosure(dest, func_id, captures)`
    AllocClosure(MirReg, hir::FuncId, Vec<MirOperand>),
    /// Dynamically call a closure: `CallClosure(dest, callee_reg, args)`
    CallClosure(MirReg, MirReg, Vec<MirOperand>),

    // --- Stage 4 additions ---
    /// Suspend execution (yield): `Suspend(yield_index, value)`
    Suspend(u32, MirOperand),
    /// Resume execution and load sent value: `Resume(dest, yield_index)`
    Resume(MirReg, u32),
    // --- Stage 12 additions ---
    /// Try enter context: `TryEnter(jmp_buf_reg)`
    TryEnter(MirReg),
    /// Set longjmp return point: `SetJmp(dest_reg, jmp_buf_reg)`
    SetJmp(MirReg, MirReg),
    /// Try exit context: `TryExit`
    TryExit,
    /// Throw a value: `Throw(val_operand)`
    Throw(MirOperand),

    // --- Class global variable support ---
    /// Load a value from a global variable: `LoadGlobal(dest, name)`
    LoadGlobal(MirReg, String),
    /// Store a value to a global variable: `StoreGlobal(name, val_operand)`
    StoreGlobal(String, MirOperand),
}

// ---------------------------------------------------------------------------
// Basic blocks & functions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub id: BlockId,
    pub instrs: Vec<MirInstr>,
}

#[derive(Debug, Clone)]
pub struct MirFunction {
    pub name: String,
    pub params: Vec<(MirReg, String)>,
    pub blocks: Vec<BasicBlock>,
    pub next_reg: MirReg,
    pub next_block: BlockId,
    pub is_generator: bool,
    pub is_async: bool,
    pub num_yield_points: u32,
    pub yield_saves: Vec<Vec<MirReg>>,
}

#[derive(Debug, Clone)]
pub struct MirModule {
    /// User-defined functions.
    pub functions: Vec<MirFunction>,
    /// Synthetic function containing all top-level statements.
    pub main_body: MirFunction,
    /// All class declarations collected during lowering.
    pub classes: std::collections::HashMap<String, hir::HirClass>,
    /// Mapping from HIR FuncId to lowered MIR function name.
    pub func_id_to_name: std::collections::HashMap<hir::FuncId, String>,
}
