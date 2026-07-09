//! MIR type definitions — three-address, basic-block IR.

pub use crate::lower::builtins::BuiltinFn;

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

    // ── bitwise ────────────────────────────────────────────────────────────
    BitAnd(MirReg, MirOperand, MirOperand),
    BitOr(MirReg, MirOperand, MirOperand),
    BitXor(MirReg, MirOperand, MirOperand),
    Shl(MirReg, MirOperand, MirOperand),
    Shr(MirReg, MirOperand, MirOperand),
    UShr(MirReg, MirOperand, MirOperand),
    BitNot(MirReg, MirOperand),

    // ── data movement ──────────────────────────────────────────────────────
    Move(MirReg, MirOperand),
    ForceOwnedTag(MirReg),

    // ── calls ──────────────────────────────────────────────────────────────
    CallDirect(MirReg, String, Vec<MirOperand>),
    CallPure(MirReg, String, Vec<MirOperand>),
    CallBuiltin(MirReg, BuiltinFn, Vec<MirOperand>),

    // ── control flow ───────────────────────────────────────────────────────
    Branch(MirOperand, BlockId, BlockId),
    Jump(BlockId),
    Return(Option<MirOperand>),

    // --- Stage 2 additions ---
    /// Allocate an object with a given shape (fallback): `Alloc(dest, class_name)`
    Alloc(MirReg, String),
    /// Allocate an object with a given shape as Shared (RC=1): `AllocShared(dest, class_name)`
    AllocShared(MirReg, String),
    /// Allocate a strictly acyclic object with a given shape: `AllocAcyclic(dest, class_name)`
    AllocAcyclic(MirReg, String),
    /// Allocate a strictly acyclic object with a given shape as Shared: `AllocSharedAcyclic(dest, class_name)`
    AllocSharedAcyclic(MirReg, String),
    /// Allocate an object with a given shape as Owned (no RC): `AllocOwned(dest, class_name)`
    AllocOwned(MirReg, String),
    /// Allocate an object with a given shape on the Stack: `AllocStack(dest, class_name)`
    AllocStack(MirReg, String),
    
    // --- Stage 4: Arena Layer ---
    /// Create an arena for a region: `ArenaCreate(region_id, initial_capacity)`
    ArenaCreate(u32, u64),
    /// Allocate an object inside an arena: `AllocArena(dest, class_name, region_id)`
    AllocArena(MirReg, String, u32),
    /// Destroy an arena (free all memory in bulk): `ArenaDestroy(region_id)`
    ArenaDestroy(u32),
    /// Call drop_fn without freeing: `CallDropFnOnly(reg)`
    CallDropFnOnly(MirReg),
    
    // --- Stage 5: RAII Scope Guards ---
    /// Push a scope guard for deterministic resource release
    ScopeGuardPush { scope_id: u32, reg: MirReg, release_fn: String },
    /// Cancel a previously armed scope guard
    ScopeGuardCancel { scope_id: u32, reg: MirReg },
    /// Flush all scope guards down to (and including) the target scope
    ScopeGuardFlushTo { current_scope: u32, target_scope: u32 },
    
    /// Increment reference count: `RcInc(reg)`
    RcInc(MirReg),
    /// Decrement reference count: `RcDec(reg)`
    RcDec(MirReg),
    /// Increment reference count (deferred via thread-local buffer): `RcIncDeferred(reg)`
    RcIncDeferred(MirReg),
    /// Decrement reference count (deferred via thread-local buffer): `RcDecDeferred(reg)`
    RcDecDeferred(MirReg),
    /// Flush all deferred reference count operations
    FlushRcDelta,
    /// Drop a value (call drop_fn and free memory): `Drop(reg)`
    Drop(MirReg),
    /// Drop a value on the stack (call drop_fn but do NOT free memory): `DropStack(reg)`
    DropStack(MirReg),
    
    /// Non-owning reference: `Borrow(dest, src)`
    Borrow(MirReg, MirReg),
    /// Mutable non-owning reference: `BorrowMut(dest, src)`
    BorrowMut(MirReg, MirReg),

    /// Load a field from an object at a static index: `LoadField(dest, obj_reg, index)`
    LoadField(MirReg, MirReg, u32),
    /// Store a value to a field of an object at a static index: `StoreField(obj_reg, index, val_operand)`
    StoreField(MirReg, u32, MirOperand),
    /// Store a value to a field with RC awareness (RcDec old, RcInc new if not moved): `StoreSharedField(obj_reg, index, val_operand, is_moved)`
    StoreSharedField(MirReg, u32, MirOperand, bool),
    /// Call a method via vtable: `CallVTable(dest, obj_reg, method_index, args)`
    CallVTable(MirReg, MirReg, u32, Vec<MirOperand>),
    /// Dynamic property read: `LoadProp(dest, obj_reg, property_name)`
    LoadProp(MirReg, MirReg, String),
    /// Dynamic property write: `StoreProp(obj_reg, property_name, val_operand, is_moved)`
    StoreProp(MirReg, String, MirOperand, bool),
    /// Dynamic property delete: `DeleteProp(dest, obj_reg, prop_operand)`
    DeleteProp(MirReg, MirOperand, MirOperand),
    // --- Stage 3 additions ---
    /// Allocate a closure object: `AllocClosure(dest, func_id, captures)`
    AllocClosure(MirReg, hir::FuncId, Vec<MirOperand>),
    AllocSharedClosure(MirReg, hir::FuncId, Vec<MirOperand>),
    AllocOwnedClosure(MirReg, hir::FuncId, Vec<MirOperand>),
    /// Dynamically call a closure: `CallClosure(dest, callee_reg, args)`
    CallClosure(MirReg, MirReg, Vec<MirOperand>),

    // --- Stage 4 additions ---
    /// Suspend execution (yield): `Suspend(yield_index, value)`
    Suspend(u32, MirOperand),
    /// Resume execution and load sent value: `Resume(dest, yield_index)`
    Resume(MirReg, u32),
    // --- Stage 12 additions ---
    /// Try enter context: `TryEnter { scope_id, catch_target }`
    TryEnter { scope_id: u32, catch_target: BlockId },
    /// Try exit context: `TryExit`
    TryExit,
    /// Throw a value: `Throw(val_operand)`
    Throw(MirOperand),
    /// Re-throw an exception: `Rethrow(exn_reg)`
    Rethrow(MirReg),
    /// Landing pad for exception cleanup: `LandingPad { exn_reg, is_cleanup }`
    LandingPad { exn_reg: MirReg, is_cleanup: bool },
    /// Extract exception from landing pad: `ExtractException { dest, lp_reg }`
    ExtractException { dest: MirReg, lp_reg: MirReg },

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
    pub exception_scopes: Vec<(u32, BlockId)>,
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
