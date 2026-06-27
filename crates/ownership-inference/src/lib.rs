pub mod rc_elision;
pub mod alias_graph;
pub mod escape;
pub mod classify;
pub mod region;
pub mod acyclic;

pub mod liveness;

use mir::{MirModule, MirFunction};
use mir::types::{MirInstr, MirOperand};

pub fn run_ownership_analysis(module: &mut MirModule) {
    // Collect class sizes
    let mut class_sizes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (name, class) in &module.classes {
        // Size = (1 vtable pointer + number of fields) * 8 bytes
        // Approximation: we don't traverse the whole hierarchy here unless we need to.
        // Let's just use a safe estimation or just count the local fields.
        // Actually, let's just do a simple upper bound: 8 * (1 + class.fields.len())
        class_sizes.insert(name.clone(), 8 * (1 + class.fields.len()));
    }

    let acyclic_classes = acyclic::compute_acyclic_classes(module);

    for func in module.functions.iter_mut() {
        analyze_function(func, &class_sizes, &acyclic_classes);
    }
    analyze_function(&mut module.main_body, &class_sizes, &acyclic_classes);
}

fn analyze_function(func: &mut MirFunction, class_sizes: &std::collections::HashMap<String, usize>, acyclic_classes: &std::collections::HashSet<String>) {
    let ag = alias_graph::build_alias_graph(func);
    let ea = escape::run_escape_analysis(func);
    let mut classes = classify::classify_registers(func, &ag, &ea);
    let liveness = liveness::run_liveness_analysis(func);

    // Collect which registers are actual object allocations (destinations of Alloc instructions).
    // Only these registers should receive Drop/DropStack/RcDec instructions.
    let mut alloc_regs: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut deferred_regs: std::collections::HashSet<u32> = std::collections::HashSet::new();
    for block in &func.blocks {
        for instr in &block.instrs {
            if let MirInstr::Alloc(dest, _) = instr {
                alloc_regs.insert(*dest);
            }
        }
    }

    let regions = region::run_region_inference(func, &ea, &alloc_regs);
    let mut active_regions: std::collections::HashSet<u32> = std::collections::HashSet::new();

    // 1. Rewrite Alloc instructions based on MemoryClass and RegionMap
    for block in &mut func.blocks {
        for instr in &mut block.instrs {
            if let MirInstr::Alloc(dest, class_name) = instr {
                let mut mem_class = classes.get_class(*dest);
                
                // Upgrade Owned to Arena if it's in a region
                if let Some(&region_id) = regions.allocations.get(dest) {
                    // It's in an arena
                    mem_class = classify::MemoryClass::Arena(region_id);
                    classes.set_class(*dest, mem_class);
                    active_regions.insert(region_id);
                }
                
                // DISABLED (Phase 6 — pending escape-analysis integration):
                // Promoting Owned → Stack based on size alone is unsound:
                // objects returned from a function would become dangling stack
                // pointers (same class of bug as the disabled Arena Strategy 1).
                if mem_class == classify::MemoryClass::Owned {
                    if !ea.prevents_stack(*dest) {
                        if let Some(&size) = class_sizes.get(class_name) {
                            if size <= 256 {
                                mem_class = classify::MemoryClass::Stack;
                                classes.set_class(*dest, mem_class);
                            }
                        }
                    }
                }

                match mem_class {
                    classify::MemoryClass::Arena(region_id) => {
                        *instr = MirInstr::AllocArena(*dest, class_name.clone(), region_id);
                    }
                    classify::MemoryClass::Stack | classify::MemoryClass::Primitive => {
                        *instr = MirInstr::AllocStack(*dest, class_name.clone());
                    }
                    classify::MemoryClass::Owned => {
                        *instr = MirInstr::AllocOwned(*dest, class_name.clone());
                    }
                    classify::MemoryClass::Shared => {
                        let is_acyclic = acyclic_classes.contains(class_name);
                        if is_acyclic {
                            *instr = MirInstr::AllocSharedAcyclic(*dest, class_name.clone());
                        } else {
                            *instr = MirInstr::AllocShared(*dest, class_name.clone());
                        }
                    }
                }
            }
        }
    }

    // 1.5. Propagate Arena class through Move instructions.
    // If Move(dest, src) and src is Arena-classified, dest must also be Arena
    // to prevent spurious RcInc/RcDec on arena-allocated objects.
    let mut changed = true;
    while changed {
        changed = false;
        for block in &func.blocks {
            for instr in &block.instrs {
                if let MirInstr::Move(dest, MirOperand::Reg(src)) = instr {
                    let src_class = classes.get_class(*src);
                    if let classify::MemoryClass::Arena(region_id) = src_class {
                        if classes.get_class(*dest) != classify::MemoryClass::Arena(region_id) {
                            classes.set_class(*dest, classify::MemoryClass::Arena(region_id));
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    let entry_block_id = func.blocks.first().map(|b| b.id).unwrap_or(0);
    for block in &mut func.blocks {
        if let Some(b_last_uses) = liveness.last_uses.get(&block.id) {
            let mut inserts: std::collections::HashMap<usize, Vec<u32>> = b_last_uses.clone();

            let mut new_instrs = Vec::new();

            // 1. Insert edge drops at the START of the block
            if let Some(drops) = liveness.edge_drops.get(&block.id) {
                // // println!("Edge drops for block {}: {:?}", block.id, drops);
                for &reg in drops {
                    // Caller-owned semantics: do not drop function parameters
                    if func.params.iter().any(|(r, _)| *r == reg) {
                        continue;
                    }
                    let class = classes.get_class(reg);
                    // // println!("  class for {}: {:?}", reg, class);
                    match class {
                        classify::MemoryClass::Arena(_) | classify::MemoryClass::Primitive => {}
                        classify::MemoryClass::Stack => {
                            new_instrs.push(MirInstr::DropStack(reg));
                        }
                        classify::MemoryClass::Shared | classify::MemoryClass::Owned => {
                            new_instrs.push(MirInstr::RcDec(reg)); println!("Emitting RcDec for {}", reg);
                        }
                    }
                }
            }

            let len = block.instrs.len();
            for (idx, mut instr) in block.instrs.drain(..).enumerate() {
                let mut transferred_regs = Vec::new();
                // Detect Move Semantics for property assignments
                if let Some(regs_to_drop) = inserts.get(&idx) {
                    // println!("Instruction {}: regs_to_drop={:?}", idx, regs_to_drop);
                    if let MirInstr::StoreProp(_, _, MirOperand::Reg(val_reg), ref mut is_moved) = &mut instr {
                        if regs_to_drop.contains(val_reg) {
                            *is_moved = true;
                            transferred_regs.push(*val_reg); println!("Moved and transferred: {}", val_reg);
                        }
                    } else if let MirInstr::StoreSharedField(_, _, MirOperand::Reg(val_reg), ref mut is_moved) = &mut instr {
                        if regs_to_drop.contains(val_reg) {
                            *is_moved = true;
                            transferred_regs.push(*val_reg); println!("Moved and transferred: {}", val_reg);
                        }
                    } else if let MirInstr::Move(_, src) = &mut instr {
                        if let MirOperand::Reg(val_reg) = src {
                            if regs_to_drop.contains(val_reg) {
                                match classes.get_class(*val_reg) {
                                    classify::MemoryClass::Owned => {
                                        transferred_regs.push(*val_reg); println!("Moved and transferred: {}", val_reg);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }

                let is_terminator = idx == len - 1 && matches!(instr, MirInstr::Jump(_) | MirInstr::Branch(..) | MirInstr::Return(_) | MirInstr::Throw(_));
                
                let mut saved_terminator = None;
                if is_terminator {
                    saved_terminator = Some(instr.clone());
                } else if let MirInstr::AllocClosure(_, _, captures) = &instr {
                    new_instrs.push(instr.clone());
                    for cap in captures {
                        if let MirOperand::Reg(val_reg) = cap {
                            match classes.get_class(*val_reg) {
                                classify::MemoryClass::Shared => {
                                    new_instrs.push(MirInstr::RcInc(*val_reg));
                                }
                                _ => {}
                            }
                        }
                    }
                    // skip the rest of the generic RcInc logic
                } else {
                    let mut is_deferred = false;
                    let maybe_rc_inc = if let MirInstr::Move(dest, src) = &instr {
                        if let MirOperand::Reg(_) = src {
                            match classes.get_class(*dest) {
                                classify::MemoryClass::Shared => {
                                    Some(MirInstr::RcInc(*dest))
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else if let MirInstr::StoreField(_, _, MirOperand::Reg(val_reg)) = &instr {
                        let cls = classes.get_class(*val_reg);
                        // println!("StoreField for reg {}: class={:?}", val_reg, cls);
                        match cls {
                            classify::MemoryClass::Shared => {
                                Some(MirInstr::RcInc(*val_reg))
                            }
                            _ => None,
                        }
                    } else if let MirInstr::StoreSharedField(_, _, MirOperand::Reg(val_reg), is_moved) = &instr {
                        if !is_moved {
                            match classes.get_class(*val_reg) {
                                classify::MemoryClass::Shared => {
                                    Some(MirInstr::RcInc(*val_reg))
                                }
                                _ => None,
                            }
                        } else {
                            None
                        }
                    } else if let MirInstr::LoadField(dest, obj_reg, _) | MirInstr::LoadProp(dest, obj_reg, _) = &instr {
                        match classes.get_class(*dest) {
                            classify::MemoryClass::Shared => {
                                if classes.get_class(*obj_reg) == classify::MemoryClass::Shared {
                                    is_deferred = true;
                                }
                                Some(MirInstr::RcInc(*dest))
                            }
                            _ => None,
                        }
                    } else if let MirInstr::LoadGlobal(dest, _) = &instr {
                        match classes.get_class(*dest) {
                            classify::MemoryClass::Shared => {
                                is_deferred = true;
                                Some(MirInstr::RcInc(*dest))
                            }
                            _ => None,
                        }
                    } else {
                        None
                    };
                    let is_store = matches!(instr, MirInstr::StoreField(..) | MirInstr::StoreSharedField(..) | MirInstr::StoreProp(..));
                    
                    if is_store {
                        if let Some(mut rc_inc) = maybe_rc_inc {
                            if is_deferred {
                                if let MirInstr::RcInc(reg) = rc_inc {
                                    deferred_regs.insert(reg);
                                    rc_inc = MirInstr::RcIncDeferred(reg);
                                }
                            }
                            new_instrs.push(rc_inc);
                        }
                        new_instrs.push(instr);
                    } else {
                        new_instrs.push(instr);
                        if let Some(mut rc_inc) = maybe_rc_inc {
                            if is_deferred {
                                if let MirInstr::RcInc(reg) = rc_inc {
                                    deferred_regs.insert(reg);
                                    rc_inc = MirInstr::RcIncDeferred(reg);
                                }
                            }
                            new_instrs.push(rc_inc);
                        }
                    }
                }

                let mut term_rc_inc = None;
                if is_terminator {
                    if let Some(term) = &saved_terminator {
                        if matches!(term, MirInstr::Return(_) | MirInstr::Throw(_) | MirInstr::Suspend(..)) {
                            let mut used_reg = None;
                            match term {
                                MirInstr::Return(Some(MirOperand::Reg(r))) => used_reg = Some(*r),
                                MirInstr::Throw(MirOperand::Reg(r)) => used_reg = Some(*r),
                                MirInstr::Suspend(_, MirOperand::Reg(r)) => used_reg = Some(*r),
                                _ => {}
                            }
                            if let Some(r) = used_reg {
                                match classes.get_class(r) {
                                    classify::MemoryClass::Shared => {
                                        term_rc_inc = Some(MirInstr::RcInc(r));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                if let Some(inc) = term_rc_inc {
                    new_instrs.push(inc);
                }

                if let Some(regs_to_drop) = inserts.get(&idx) {
                    for &reg in regs_to_drop {
                        if transferred_regs.contains(&reg) {
                            continue;
                        }

                        // Caller-owned semantics: do not drop function parameters
                        if func.params.iter().any(|(r, _)| *r == reg) {
                            continue;
                        }

                        let mut used_reg = None;
                        if is_terminator {
                            if let Some(term) = &saved_terminator {
                                match term {
                                    MirInstr::Return(Some(MirOperand::Reg(r))) => used_reg = Some(*r),
                                    MirInstr::Throw(MirOperand::Reg(r)) => used_reg = Some(*r),
                                    MirInstr::Suspend(_, MirOperand::Reg(r)) => used_reg = Some(*r),
                                    _ => {}
                                }
                            }
                        }

                        match classes.get_class(reg) {
                            classify::MemoryClass::Arena(_) | classify::MemoryClass::Primitive => {
                                // Arena objects and primitives don't get individual drops
                            }
                            classify::MemoryClass::Stack => {
                                new_instrs.push(MirInstr::DropStack(reg));
                            }
                            classify::MemoryClass::Shared => {
                                // Drop any register that holds a Shared object at its last use
                                if deferred_regs.contains(&reg) || used_reg == Some(reg) {
                                    new_instrs.push(MirInstr::RcDecDeferred(reg));
                                } else {
                                    new_instrs.push(MirInstr::RcDec(reg)); println!("Emitting RcDec for {}", reg);
                                }
                            }
                            classify::MemoryClass::Owned => {
                                new_instrs.push(MirInstr::Drop(reg)); println!("Emitting Drop for {}", reg);
                            }
                        }
                    }
                }
                
                // If it's a terminator and there are active regions, we might need to destroy them
                if is_terminator {
                    if let Some(term) = saved_terminator {
                        if matches!(term, MirInstr::Return(_) | MirInstr::Throw(_)) {
                            // Destroy arenas before returning
                            for &region_id in &active_regions {
                                new_instrs.push(MirInstr::ArenaDestroy(region_id));
                            }
                            new_instrs.push(MirInstr::FlushRcDelta);
                        } else if matches!(term, MirInstr::Suspend(..)) {
                            new_instrs.push(MirInstr::FlushRcDelta);
                        }
                        new_instrs.push(term);
                    }
                }
            }
            
            // If this is the entry block, insert ArenaCreate at the beginning
            if block.id == entry_block_id && !active_regions.is_empty() {
                let mut entry_instrs = Vec::new();
                for &region_id in &active_regions {
                    entry_instrs.push(MirInstr::ArenaCreate(region_id, 4096)); // default capacity
                }
                entry_instrs.extend(new_instrs);
                block.instrs = entry_instrs;
            } else {
                block.instrs = new_instrs;
            }
        }
    }

    if func.name.contains("testDestructuring") {
        for block in &func.blocks {
            // println!("Block {}:", block.id);
            for instr in &block.instrs {
                // println!("  {:?}", instr);
            }
        }
    }

    rc_elision::run_rc_elision(func);
}
