use mir::types::{MirModule, MirInstr};
use std::collections::{HashMap, HashSet};

pub fn analyze_purity(module: &mut MirModule) {
    // Initial pass: assume everything is pure, then disqualify
    let mut is_pure_map: HashMap<String, bool> = HashMap::new();
    for f in module.functions.iter() {
        is_pure_map.insert(f.name.clone(), true);
    }
    is_pure_map.insert(module.main_body.name.clone(), true);
    
    let mut changed = true;
    while changed {
        changed = false;
        
        let mut check_funcs = Vec::new();
        for f in module.functions.iter() {
            check_funcs.push((f.name.clone(), f.blocks.clone()));
        }
        check_funcs.push((module.main_body.name.clone(), module.main_body.blocks.clone()));
        
        for (name, blocks) in check_funcs {
            if !is_pure_map[&name] {
                continue;
            }
            
            let mut is_pure = true;
            for block in &blocks {
                for instr in &block.instrs {
                    match instr {
                        MirInstr::StoreGlobal(..) |
                        MirInstr::Throw(_) |
                        MirInstr::Rethrow(_) |
                        MirInstr::DeleteProp(..) |
                        MirInstr::CallBuiltin(..) | // Assume all builtins are impure for now
                        MirInstr::CallClosure(..) |
                        MirInstr::CallVTable(..) |
                        MirInstr::Suspend(..) |
                        MirInstr::TryEnter { .. } |
                        MirInstr::TryExit => {
                            is_pure = false;
                            break;
                        }
                        MirInstr::CallDirect(_, target, _) | MirInstr::CallPure(_, target, _) => {
                            if let Some(&target_pure) = is_pure_map.get(target) {
                                if !target_pure {
                                    is_pure = false;
                                    break;
                                }
                            } else {
                                // External function (like console.log), assume impure
                                is_pure = false;
                                break;
                            }
                        }
                        _ => {}
                    }
                }
                if !is_pure {
                    break;
                }
            }
            
            if !is_pure {
                is_pure_map.insert(name.clone(), false);
                changed = true;
            }
        }
    }
    
    // Now, apply the purity flag to CallDirect instructions in all functions
    for f in module.functions.iter_mut() {
        for block in &mut f.blocks {
            for instr in &mut block.instrs {
                if let MirInstr::CallDirect(dest, target, args) = instr {
                    if let Some(&true) = is_pure_map.get(target) {
                        *instr = MirInstr::CallPure(*dest, target.clone(), args.clone());
                    }
                }
            }
        }
    }
    for block in &mut module.main_body.blocks {
        for instr in &mut block.instrs {
            if let MirInstr::CallDirect(dest, target, args) = instr {
                if let Some(&true) = is_pure_map.get(target) {
                    *instr = MirInstr::CallPure(*dest, target.clone(), args.clone());
                }
            }
        }
    }
}
