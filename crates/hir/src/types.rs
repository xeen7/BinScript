use serde::{Serialize, Deserialize};

mod serde_arc_u8 {
    use std::sync::Arc;
    use serde::{Serialize, Deserialize, Serializer, Deserializer};

    pub fn serialize<S>(val: &Arc<[u8]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        val.as_ref().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Arc<[u8]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        Ok(Arc::from(vec))
    }
}

/// Unique identifier for a variable binding within a compilation unit.
pub type BindingId = u32;

/// Unique identifier for a function within a compilation unit.
pub type FuncId = u32;

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
    Undefined,
}

// ---------------------------------------------------------------------------
// Operators
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BinOp {
    // Arithmetic
    Add, Sub, Mul, Div, Mod, Exp,
    // Comparison
    Eq, Ne, StrictEq, StrictNe,
    Lt, Le, Gt, Ge, In,
    // Logical
    And, Or, NullishCoalescing,
    // Bitwise
    BitAnd, BitOr, BitXor,
    Shl, Shr, UShr,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UnaryOp {
    Plus,
    Neg,
    Not,
    BitNot,
    Typeof,
    Void,
}

// ---------------------------------------------------------------------------
// HIR Expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HirExpr {
    /// A literal value.
    Lit(Literal),
    /// Reference to a locally-bound variable.
    Var(BindingId),
    /// Binary operation.
    BinOp(BinOp, Box<HirExpr>, Box<HirExpr>),
    /// Unary operation.
    UnaryOp(UnaryOp, Box<HirExpr>),
    /// Function call.
    Call { callee: Box<HirExpr>, args: Vec<HirExpr> },
    /// Assignment to a variable (expression form, returns the assigned value).
    Assign { target: BindingId, value: Box<HirExpr> },
    /// Ternary / conditional expression.
    Ternary { cond: Box<HirExpr>, then_expr: Box<HirExpr>, else_expr: Box<HirExpr> },
    /// Comma-separated expression sequence (returns last).
    Seq(Vec<HirExpr>),
    /// Reference to a global name (e.g. `console`, `Math`).
    GlobalRef(String),
    /// Method call on a known global object (e.g. `console.log(...)`).
    MemberCall { object: String, method: String, args: Vec<HirExpr> },
    // --- Stage 2 additions ---
    /// Property read: `object.property`
    MemberGet { object: Box<HirExpr>, property: String },
    /// Property write: `object.property = value`
    MemberSet { object: Box<HirExpr>, property: String, value: Box<HirExpr> },
    /// Instantiate a class: `new class_name(...args)`
    New { class_name: String, args: Vec<HirExpr> },
    /// Instanceof check: `expr instanceof class_name`
    InstanceOf { expr: Box<HirExpr>, class_name: String },
    /// Method call: `object.method(...args)`
    MethodCall { object: Box<HirExpr>, method: String, args: Vec<HirExpr> },
    // --- Stage 3 additions ---
    /// Instantiation of a closure: `Closure(func_id, captures)`
    Closure { func_id: FuncId, captures: Vec<BindingId> },
    // --- Stage 4 additions ---
    /// Yield expression inside a generator: `yield expr` or `yield* expr`
    Yield { arg: Option<Box<HirExpr>>, delegate: bool },
    /// Await expression: `await expr`
    Await(Box<HirExpr>),
    // --- Stage 6 additions ---
    /// Raw JSON tape bytes
    #[serde(with = "serde_arc_u8")]
    JsonTape(std::sync::Arc<[u8]>),
    // --- Stage 11 additions ---
    /// Array literal: `[a, b, c]`
    ArrayLit(Vec<HirExpr>),
    /// Element/Index read: `object[index]`
    IndexGet { object: Box<HirExpr>, index: Box<HirExpr> },
    /// Element/Index write: `object[index] = value`
    IndexSet { object: Box<HirExpr>, index: Box<HirExpr>, value: Box<HirExpr> },
    /// Spread element: `...expr`
    Spread(Box<HirExpr>),
    /// Delete property: `delete object[property]`
    DeleteProp { object: Box<HirExpr>, property: Box<HirExpr> },
}

// ---------------------------------------------------------------------------
// HIR Statements
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirSwitchCase {
    pub test: Option<HirExpr>,
    pub consequent: Vec<HirStmt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HirStmt {
    /// Expression statement.
    Expr(HirExpr),
    /// Variable declaration (`let` / `const`).
    Let { binding: BindingId, name: String, init: Option<HirExpr> },
    /// Assignment statement.
    Assign { target: BindingId, value: HirExpr },
    /// Switch statement.
    Switch { discriminant: HirExpr, cases: Vec<HirSwitchCase> },
    /// If / else.
    If { cond: HirExpr, then_body: Vec<HirStmt>, else_body: Option<Vec<HirStmt>> },
    /// While loop.
    While { cond: HirExpr, body: Vec<HirStmt> },
    /// Do-while loop.
    DoWhile { body: Vec<HirStmt>, cond: HirExpr },
    /// C-style for loop.
    For {
        init: Option<Box<HirStmt>>,
        cond: Option<HirExpr>,
        update: Option<HirExpr>,
        body: Vec<HirStmt>,
    },
    /// For-Of loop: `for (let x of iter) { ... }`
    ForOf {
        /// The variable declaration (or assignment target). Usually a `Let` stmt or an `Assign` stmt.
        left: Box<HirStmt>,
        /// The iterable expression
        right: HirExpr,
        /// Loop body
        body: Vec<HirStmt>,
        /// Whether this is a 'for await' loop
        is_await: bool,
    },
    /// Return from function.
    Return(Option<HirExpr>),
    /// Break out of the loop/block.
    Break(Option<String>),
    /// Continue to the next iteration of the loop.
    Continue(Option<String>),
    /// Labeled statement.
    Labeled { label: String, body: Box<HirStmt> },
    /// Block of statements.
    Block(Vec<HirStmt>),
    /// Function declaration.
    FuncDecl {
        id: FuncId,
        name: String,
        params: Vec<(BindingId, String)>,
        body: Vec<HirStmt>,
    },
    /// Throw statement.
    Throw(HirExpr),
    /// Try statement.
    Try {
        body: Vec<HirStmt>,
        catch_param: Option<(BindingId, String)>,
        catch_body: Vec<HirStmt>,
        finally_body: Option<Vec<HirStmt>>,
    },
}

// ---------------------------------------------------------------------------
// Top-level structures
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirMethod {
    pub name: String,
    pub func_id: FuncId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirClass {
    pub name: String,
    pub super_name: Option<String>,
    pub fields: Vec<String>,
    pub methods: Vec<HirMethod>,
    pub getters: Vec<String>,
    pub setters: Vec<String>,
    pub static_getters: Vec<String>,
    pub static_setters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirFunction {
    pub id: FuncId,
    pub name: String,
    pub params: Vec<(BindingId, String)>,
    pub body: Vec<HirStmt>,
    pub captures: Vec<BindingId>,
    pub is_generator: bool,
    pub is_async: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReExport {
    pub src: String,
    pub local: String,
    pub exported: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleExports {
    /// Named exports: export name → BindingId
    pub named: std::collections::HashMap<String, BindingId>,
    /// Default export (if any)
    pub default: Option<BindingId>,
    /// Exported function declarations: export name → FuncId
    pub functions: std::collections::HashMap<String, FuncId>,
    /// Exported class declarations: export name → class name
    pub classes: std::collections::HashMap<String, String>,
    /// Re-exports: `export { foo } from './other'`
    pub re_exports: Vec<ReExport>,
    /// Export alls: `export * from './other'`
    pub export_alls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HirModule {
    /// Top-level statements (executed sequentially as the program entry).
    pub stmts: Vec<HirStmt>,
    /// All function declarations collected during lowering.
    pub functions: Vec<HirFunction>,
    /// All class declarations collected during lowering.
    pub classes: std::collections::HashMap<String, HirClass>,
    pub capture_cells: std::collections::HashSet<BindingId>,
    pub next_binding_id: BindingId,
    pub next_func_id: FuncId,
    /// Exports of this module.
    pub exports: ModuleExports,
}

