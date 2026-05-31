//! `LowerCtx` struct definition and core scope/bookkeeping helpers.

use std::collections::{HashMap, HashSet};
use crate::types::*;

/// Internal lowering context threaded through the entire AST walk.
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
    pub(crate) fn new() -> Self {
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
}
