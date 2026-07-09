use mir::types::MirReg;
use crate::escape::EscapeAnalysis;
use std::collections::{HashMap, HashSet};
use mir::MirFunction;

pub type RegionId = u32;

pub struct RegionMap {
    pub allocations: HashMap<MirReg, RegionId>,
    pub next_region_id: RegionId,
}

impl RegionMap {
    pub fn new() -> Self {
        Self {
            allocations: HashMap::new(),
            next_region_id: 1, // 0 is reserved/invalid
        }
    }
}

/// Runs region inference on a function.
/// For Phase 4.1, we only implement Strategy 1:
/// If EVERY allocation in the function does not escape,
/// they can all share a single function-scoped Arena.
pub fn run_region_inference(
    _func: &MirFunction,
    escape: &EscapeAnalysis,
    alloc_regs: &HashSet<MirReg>,
) -> RegionMap {
    let mut map = RegionMap::new();
    
    // Async and Generator functions suspend execution. 
    // ArenaCreate pointers are not saved in the coroutine state, causing LLVM SSA dominance errors.
    if _func.is_async || _func.is_generator {
        return map;
    }
    
    // Check if ALL allocations are local
    let mut all_local = true;
    for &reg in alloc_regs {
        if escape.does_escape(reg) {
            all_local = false;
            break;
        }
    }

    // Strategy 1 is now re-enabled, as escape analysis tracks assignments and dataflow securely.
    
    // Strategy 1: Function-scope arena
    if all_local && !alloc_regs.is_empty() {
        let region_id = map.next_region_id;
        map.next_region_id += 1;
        
        for &reg in alloc_regs {
            map.allocations.insert(reg, region_id);
        }
    }
    
    map
}
