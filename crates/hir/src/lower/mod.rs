//! OXC AST → HIR lowering.
//!
//! Walks the OXC JavaScript AST and produces
//! a simplified HIR suitable for further lowering to MIR.

use oxc::ast::ast::*;

use diagnostics::CompileResult;

use crate::types::*;

// Submodules
pub(crate) mod context;
mod module;
mod patterns;
pub(crate) mod operators;
pub(crate) mod capture;
mod expr;
mod stmt;

pub(crate) use context::LowerCtx;
pub(crate) use operators::{conv_bin_op, conv_unary_op, conv_logical_op};

/// Lowers an OXC `Program` into an `HirModule`.
pub fn lower_module(program: &Program) -> CompileResult<HirModule> {
    let mut ctx = LowerCtx::new();
    ctx.lower_module(program)
}

/// Lower a module with pre-resolved import bindings injected into scope.
pub fn lower_module_with_imports(
    program: &Program,
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

    ctx.lower_module(program)
}
