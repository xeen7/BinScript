use std::collections::{HashMap, HashSet};
use crate::types::{BlockId, MirFunction, MirInstr, MirReg};
use crate::pattern_match;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReleaseState {
    Unreachable,
    Acquired,
    Released,
    MaybeReleased,
}

impl ReleaseState {
    pub fn join(self, other: ReleaseState) -> ReleaseState {
        match (self, other) {
            (ReleaseState::Unreachable, x) | (x, ReleaseState::Unreachable) => x,
            (ReleaseState::Released, ReleaseState::Released) => ReleaseState::Released,
            (ReleaseState::Acquired, ReleaseState::Acquired) => ReleaseState::Acquired,
            (ReleaseState::MaybeReleased, _) | (_, ReleaseState::MaybeReleased) => ReleaseState::MaybeReleased,
            _ => ReleaseState::MaybeReleased, // Acquired + Released = MaybeReleased
        }
    }
}

pub struct DraContext<'a> {
    pub function: &'a MirFunction,
    pub resource: MirReg,
    pub aliases: &'a HashMap<MirReg, MirReg>,
}

pub fn run_dra(ctx: &DraContext) -> HashMap<BlockId, ReleaseState> {
    let mut in_states: HashMap<BlockId, ReleaseState> = HashMap::new();
    let mut out_states: HashMap<BlockId, ReleaseState> = HashMap::new();
    
    // Initialize
    for block in &ctx.function.blocks {
        in_states.insert(block.id, ReleaseState::Unreachable);
        out_states.insert(block.id, ReleaseState::Unreachable);
    }
    
    // Entry block starts as Unreachable until we hit the acquisition
    in_states.insert(0, ReleaseState::Unreachable);
    
    // Build CFG predecessors
    let mut preds: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
    for block in &ctx.function.blocks {
        if let Some(instr) = block.instrs.last() {
            match instr {
                MirInstr::Jump(target) => {
                    preds.entry(*target).or_default().push(block.id);
                }
                MirInstr::Branch(_, t1, t2) => {
                    preds.entry(*t1).or_default().push(block.id);
                    preds.entry(*t2).or_default().push(block.id);
                }
                _ => {
                    if !matches!(instr, MirInstr::Return(_) | MirInstr::Throw(_)) {
                        preds.entry(block.id + 1).or_default().push(block.id);
                    }
                }
            }
        } else {
            preds.entry(block.id + 1).or_default().push(block.id);
        }
        
        // Also scan all instructions for TryEnter to add catch edges
        for instr in &block.instrs {
            if let MirInstr::TryEnter { catch_target, .. } = instr {
                preds.entry(*catch_target).or_default().push(block.id);
            }
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        
        for block in &ctx.function.blocks {
            // Join predecessors
            let mut new_in = if block.id == 0 { ReleaseState::Unreachable } else { ReleaseState::Unreachable };
            
            if let Some(predecessors) = preds.get(&block.id) {
                for &p in predecessors {
                    let pred_out = out_states[&p];
                    new_in = new_in.join(pred_out);
                }
            }
            
            in_states.insert(block.id, new_in);
            
            // Transfer function
            let mut current = new_in;
            for instr in &block.instrs {
                if let Some((reg, _)) = pattern_match::is_acquisition_call(instr) {
                    let resolved = ctx.aliases.get(&reg).copied().unwrap_or(reg);
                    if resolved == ctx.resource {
                        current = ReleaseState::Acquired;
                    }
                } else if let Some((reg, _)) = pattern_match::is_release_call(instr) {
                    let resolved = ctx.aliases.get(&reg).copied().unwrap_or(reg);
                    if resolved == ctx.resource {
                        current = ReleaseState::Released;
                    }
                }
            }
            
            if out_states[&block.id] != current {
                out_states.insert(block.id, current);
                changed = true;
            }
        }
    }
    
    out_states
}
