//! Basic scope analysis.
//!
//! For Stage 1 the scope logic is embedded directly in the lowering pass
//! (`LowerCtx::scopes`). This module is a placeholder for future expansion
//! (closure capture analysis, `this` binding analysis, etc. in Stages 3–4).

use std::collections::HashMap;
use crate::types::BindingId;

/// A single scope in the scope tree.
#[derive(Debug, Clone)]
pub struct Scope {
    pub bindings: HashMap<String, BindingId>,
    pub parent: Option<usize>,
}

/// Scope tree (currently unused — scope logic lives in `lower::LowerCtx`).
#[derive(Debug, Clone, Default)]
pub struct ScopeTree {
    pub scopes: Vec<Scope>,
}
