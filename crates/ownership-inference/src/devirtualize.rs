use mir::MirModule;
use mir::types::{MirInstr, MirOperand, MirReg};
use std::collections::HashMap;

pub fn run_devirtualize_pass(module: &mut MirModule) {
    let mut func_id_to_name = module.func_id_to_name.clone();

    let mut process_func = |func: &mut mir::MirFunction| {
        let mut reg_to_func = HashMap::new();

        for b in &mut func.blocks {
            for instr in &mut b.instrs {
                match instr {
                    MirInstr::AllocClosure(dest, func_id, _) |
                    MirInstr::AllocSharedClosure(dest, func_id, _) |
                    MirInstr::AllocOwnedClosure(dest, func_id, _) => {
                        reg_to_func.insert(*dest, *func_id);
                    }
                    MirInstr::Move(dest, MirOperand::Reg(src)) => {
                        if let Some(&fid) = reg_to_func.get(src) {
                            reg_to_func.insert(*dest, fid);
                        }
                    }
                    MirInstr::CallClosure(dest, callee_reg, args) => {
                        if let Some(&func_id) = reg_to_func.get(callee_reg) {
                            if let Some(target_name) = func_id_to_name.get(&func_id) {
                                // Convert to CallDirect
                                // We need to insert the callee_reg as the FIRST argument (__env)
                                *instr = MirInstr::CallDirect(*dest, target_name.clone(), args.clone());
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    };

    for func in &mut module.functions {
        process_func(func);
    }
    process_func(&mut module.main_body);
}
