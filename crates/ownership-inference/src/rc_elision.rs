use mir::MirFunction;
use mir::types::MirInstr;
use std::collections::HashMap;

pub fn run_rc_elision(func: &mut MirFunction) {
    for block in &mut func.blocks {
        // Map from register to the instruction index of its RcInc
        let mut elidable_incs: HashMap<u32, usize> = HashMap::new();
        // Set of instruction indices to remove
        let mut to_remove = std::collections::HashSet::new();

        for (idx, instr) in block.instrs.iter().enumerate() {
            match instr {
                MirInstr::RcInc(reg) | MirInstr::RcIncDeferred(reg) => {
                    elidable_incs.insert(*reg, idx);
                }
                MirInstr::RcDec(reg) | MirInstr::RcDecDeferred(reg) => {
                    if let Some(&inc_idx) = elidable_incs.get(reg) {
                        // Found a matching RcInc with no invalidations between them!
                        // if *reg == 8 { println!("Eliding RcDec(8) at idx {} in block {}", idx, block.id); }
                        to_remove.insert(inc_idx);
                        to_remove.insert(idx);
                        elidable_incs.remove(reg);
                    } else {
                        // A RcDec that we can't elide. This could drop the object,
                        // so it invalidates all currently pending elidable Incs.
                        elidable_incs.clear();
                    }
                }
                
                // --- Invalidation Instructions ---
                // Any instruction that could potentially drop an object, 
                // escape an object, or transfer control flow out of the block.
                MirInstr::Drop(_) |
                MirInstr::DropStack(_) |
                MirInstr::CallDirect(..) |
                MirInstr::CallBuiltin(..) |
                MirInstr::CallClosure(..) |
                MirInstr::CallVTable(..) |
                MirInstr::CallDropFnOnly(_) |
                MirInstr::ArenaDestroy(_) |
                MirInstr::StoreGlobal(..) |
                MirInstr::StoreField(..) |
                MirInstr::StoreSharedField(..) |
                MirInstr::StoreProp(..) |
                MirInstr::DeleteProp(..) |
                MirInstr::Suspend(..) |
                MirInstr::TryEnter { .. } |
                MirInstr::TryExit |
                MirInstr::Throw(_) |
                MirInstr::Rethrow(_) |
                MirInstr::FlushRcDelta => {
                    elidable_incs.clear();
                }

                // Other instructions (arithmetic, simple moves, loads, branches) 
                // do not drop objects or escape them in a way that causes drops.
                _ => {}
            }
        }

        // Apply removals
        if !to_remove.is_empty() {
            let mut new_instrs = Vec::with_capacity(block.instrs.len());
            for (idx, instr) in block.instrs.drain(..).enumerate() {
                if !to_remove.contains(&idx) {
                    new_instrs.push(instr);
                }
            }
            block.instrs = new_instrs;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mir::types::{BasicBlock, MirOperand};

    #[test]
    fn test_elision_simple() {
        let mut func = MirFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![BasicBlock {
                id: 0,
                exception_scopes: vec![],
                instrs: vec![
                    MirInstr::RcInc(1),
                    MirInstr::Move(2, MirOperand::Reg(1)),
                    MirInstr::RcDec(1),
                ],
            }],
            next_reg: 3,
            next_block: 1,
            is_generator: false,
            is_async: false,
            num_yield_points: 0,
            yield_saves: vec![],
        };

        run_rc_elision(&mut func);
        
        let instrs = &func.blocks[0].instrs;
        assert_eq!(instrs.len(), 1);
        assert!(matches!(instrs[0], MirInstr::Move(2, MirOperand::Reg(1))));
    }

    #[test]
    fn test_elision_blocked() {
        let mut func = MirFunction {
            name: "test".into(),
            params: vec![],
            blocks: vec![BasicBlock {
                id: 0,
                exception_scopes: vec![],
                instrs: vec![
                    MirInstr::RcInc(1),
                    MirInstr::CallDirect(2, "foo".into(), vec![]), // blocks elision
                    MirInstr::RcDec(1),
                ],
            }],
            next_reg: 3,
            next_block: 1,
            is_generator: false,
            is_async: false,
            num_yield_points: 0,
            yield_saves: vec![],
        };

        run_rc_elision(&mut func);
        
        let instrs = &func.blocks[0].instrs;
        assert_eq!(instrs.len(), 3);
    }
}
