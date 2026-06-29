use std::collections::HashMap;
use crate::types::{BlockId, MirInstr, MirModule, MirReg, MirFunction, MirOperand};
use crate::pattern_match;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RaiiSource {
    TypeRegistry,
    LifecyclePattern,
    TryFinally,
    ControlFlowCompletion,
}

#[derive(Debug, Clone)]
pub struct ResourceDescriptor {
    pub binding: MirReg,
    pub acquire_site: BlockId, 
    pub release_fn: String,
    pub source: RaiiSource,
    pub confidence: u8,
}

pub struct ResourceDescriptorTable {
    pub entries: HashMap<MirReg, ResourceDescriptor>,
}

impl ResourceDescriptorTable {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

fn detect_resources(function: &mut MirFunction, rdt: &mut ResourceDescriptorTable) {
    let mut acquired_in_try = Vec::new(); 
    let mut aliases = HashMap::new();
    let mut all_releases = HashMap::new(); // reg -> release_fn

    for (block_id, block) in function.blocks.iter().enumerate() {
        for instr in &block.instrs {
            if let MirInstr::Move(dest, MirOperand::Reg(src)) = instr {
                aliases.insert(*dest, *src); 
            }
            if let Some((reg, _verb)) = pattern_match::is_acquisition_call(instr) {
                acquired_in_try.push((block_id as u32, reg));
            }
            if let Some((reg, release_fn)) = pattern_match::is_release_call(instr) {
                let resolved = aliases.get(&reg).copied().unwrap_or(reg);
                all_releases.insert(resolved, release_fn);
            }
        }
    }

    // Now, for every acquisition, if it has a paired release somewhere, it's a resource
    for (block_id, reg) in &acquired_in_try {
        let resolved = aliases.get(reg).copied().unwrap_or(*reg);
        if let Some(release_fn) = all_releases.get(&resolved) {
            rdt.entries.insert(resolved, ResourceDescriptor {
                binding: resolved,
                acquire_site: *block_id, 
                release_fn: release_fn.clone(),
                source: RaiiSource::LifecyclePattern,
                confidence: 90,
            });
        }
    }
}

pub fn run_lifecycle_pass(module: &mut MirModule) {
    let mut rdt = ResourceDescriptorTable::new();
    
    for function in module.functions.iter_mut().chain(std::iter::once(&mut module.main_body)) {
        detect_resources(function, &mut rdt);
        
        let mut aliases = HashMap::new();
        for block in &function.blocks {
            for instr in &block.instrs {
                if let MirInstr::Move(dest, MirOperand::Reg(src)) = instr {
                    aliases.insert(*dest, *src);
                }
            }
        }

        // For each resource, run DRA
        let mut missing_release_blocks = HashMap::new(); // Resource -> blocks
        for (reg, _desc) in &rdt.entries {
            let ctx = crate::dra::DraContext {
                function,
                resource: *reg,
                aliases: &aliases,
            };
            let out_states = crate::dra::run_dra(&ctx);
            
            let mut missing = Vec::new();
            for (block_id, state) in out_states {
                let block = &function.blocks[block_id as usize];
                if let Some(last_instr) = block.instrs.last() {
                    if matches!(last_instr, MirInstr::Return(_) | MirInstr::Throw(_)) {
                        if state == crate::dra::ReleaseState::Acquired || state == crate::dra::ReleaseState::MaybeReleased {
                            missing.push(block_id);
                        }
                    }
                }
            }
            missing_release_blocks.insert(*reg, missing);
        }

        insert_guards(function, &rdt, missing_release_blocks);
        rdt.entries.clear(); 
    }
}

fn insert_guards(function: &mut MirFunction, rdt: &ResourceDescriptorTable, missing_release_blocks: HashMap<MirReg, Vec<u32>>) {
    let mut to_insert = Vec::new();
    let mut aliases = HashMap::new();
    
    for (block_id, block) in function.blocks.iter().enumerate() {
        for (i, instr) in block.instrs.iter().enumerate() {
            if let MirInstr::Move(dest, MirOperand::Reg(src)) = instr {
                aliases.insert(*dest, *src); 
            }
            if let Some((reg, _release_fn)) = pattern_match::is_release_call(instr) {
                let resolved = aliases.get(&reg).copied().unwrap_or(reg);
                if rdt.entries.contains_key(&resolved) {
                    to_insert.push((block_id as u32, i, MirInstr::ScopeGuardCancel {
                        scope_id: 1, 
                        reg: resolved, 
                    }));
                }
            } else if let Some((reg, _)) = pattern_match::is_acquisition_call(instr) {
                if let Some(desc) = rdt.entries.get(&reg) {
                    to_insert.push((block_id as u32, i + 1, MirInstr::ScopeGuardPush {
                        scope_id: 1, 
                        reg,
                        release_fn: desc.release_fn.clone(),
                    }));
                }
            } else if matches!(instr, MirInstr::Return(_) | MirInstr::Throw(_)) {
                let needs_flush = missing_release_blocks.values().any(|blocks| blocks.contains(&(block_id as u32)));
                if needs_flush {
                    to_insert.push((block_id as u32, i, MirInstr::ScopeGuardFlushTo {
                        current_scope: 1,
                        target_scope: 0,
                    }));
                }
            }
        }
    }
    
    to_insert.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));
    
    for (block_id, idx, instr) in to_insert {
        function.blocks[block_id as usize].instrs.insert(idx, instr);
    }
}
