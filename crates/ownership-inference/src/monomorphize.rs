use mir::{MirModule, MirFunction};
use mir::types::{MirInstr, MirOperand, MirReg};
use std::collections::{HashMap, HashSet};
use crate::classify::{MemoryClass, classify_registers};
use crate::alias_graph;
use crate::escape::{self, EscapeAnalysis};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SignatureArg {
    Class(MemoryClass),
    ConstBool(bool),
    ConstNum(u64),
}

impl SignatureArg {
    pub fn memory_class(&self) -> MemoryClass {
        match self {
            Self::Class(c) => *c,
            Self::ConstBool(_) | Self::ConstNum(_) => MemoryClass::Primitive,
        }
    }
    
    pub fn to_string(&self) -> String {
        match self {
            Self::Class(c) => format!("{:?}", c),
            Self::ConstBool(b) => format!("ConstBool{}", b),
            Self::ConstNum(n) => format!("ConstNum{}", n),
        }
    }
}

pub fn run_monomorphize_pass(
    module: &mut MirModule,
    module_ea: &mut HashMap<String, EscapeAnalysis>,
) -> HashMap<String, Vec<SignatureArg>> {
    let mut instantiated: HashMap<(String, Vec<SignatureArg>), String> = HashMap::new();
    let mut signatures_out = HashMap::new();
    let mut new_functions = Vec::new();

    let mut worklist: Vec<(String, Option<Vec<SignatureArg>>)> = vec![("__bs_script_main".to_string(), None)];
    let mut processed_targets = HashSet::new();

    while let Some((func_name, signature)) = worklist.pop() {
        if !processed_targets.insert((func_name.clone(), signature.clone())) {
            continue;
        }

        let mut func = if func_name == "__bs_script_main" {
            module.main_body.clone()
        } else {
            module.functions.iter().chain(new_functions.iter())
                .find(|f| f.name == func_name).unwrap().clone()
        };

        let ag = alias_graph::build_alias_graph(&func);
        let ea = if let Some(e) = module_ea.get(&func_name) {
            e.clone()
        } else {
            let orig_name = func_name.split("_sig_").next().unwrap_or(&func_name);
            module_ea.get(orig_name).cloned().unwrap_or_else(|| escape::run_escape_analysis(&func, Some(module_ea)))
        };
        
        let sig_slice = signature.as_deref();
        let mem_classes: Option<Vec<MemoryClass>> = sig_slice.map(|s| s.iter().map(|arg| arg.memory_class()).collect());
        
        let dummy_return = HashSet::new();
        let dummy_param = HashMap::new();
        let classes = classify_registers(&func, &ag, &ea, &dummy_return, &dummy_param, mem_classes.as_deref());
        
        if func_name == "__bs_main_sig_" || func_name == "__bs_main" {
            println!("DEBUG MONO: Analyzing main");
            for r in 0..func.next_reg {
                println!("  Reg({}): escapes={}, class={:?}", r, ea.does_escape(r) || ea.prevents_owned(r), classes.get_class(r));
            }
            if let Some(target_ea) = module_ea.get("__bs_storeIfFlag") {
                println!("DEBUG MONO: storeIfFlag param_escapes: {:?}", target_ea.param_escapes);
            }
        }

        for b in &mut func.blocks {
            for instr in &mut b.instrs {
                if let MirInstr::CallDirect(_, target, args) = instr {
                    let mut call_sig = Vec::new();
                    let mut name_parts = Vec::new();
                    
                    for arg in args.iter() {
                        if let MirOperand::Reg(r) = arg {
                            let mut cls = classes.get_class(*r);
                            if let MemoryClass::Arena(_) = cls {
                                cls = MemoryClass::Borrow;
                            }
                            call_sig.push(SignatureArg::Class(cls));
                            name_parts.push(format!("{:?}", cls));
                        } else if let MirOperand::ConstBool(b) = arg {
                            call_sig.push(SignatureArg::ConstBool(*b));
                            name_parts.push(format!("ConstBool{}", b));
                        } else if let MirOperand::ConstNum(f) = arg {
                            call_sig.push(SignatureArg::ConstNum(f.to_bits()));
                            name_parts.push(format!("ConstNum{}", f.to_bits()));
                        } else {
                            call_sig.push(SignatureArg::Class(MemoryClass::Primitive));
                            name_parts.push("Primitive".to_string());
                        }
                    }

                    if module.functions.iter().any(|f| f.name == *target) {
                        let sig_str = name_parts.join("_");
                        let clone_name = format!("{}_sig_{}", target, sig_str);
                        
                        if !instantiated.contains_key(&(target.clone(), call_sig.clone())) {
                            instantiated.insert((target.clone(), call_sig.clone()), clone_name.clone());
                            
                            let mut cloned_func = module.functions.iter().find(|f| f.name == *target).unwrap().clone();
                            cloned_func.name = clone_name.clone();
                            
                            // Perform Constant Propagation and DCE on the clone
                            let mut const_params = HashMap::new();
                            for (idx, (reg, _)) in cloned_func.params.iter().enumerate() {
                                if idx > 0 && (idx - 1) < call_sig.len() {
                                    match call_sig[idx - 1] {
                                        SignatureArg::ConstBool(b) => { const_params.insert(*reg, MirOperand::ConstBool(b)); }
                                        SignatureArg::ConstNum(n) => { const_params.insert(*reg, MirOperand::ConstNum(f64::from_bits(n))); }
                                        _ => {}
                                    }
                                }
                            }
                            
                            if !const_params.is_empty() {
                                for b in &mut cloned_func.blocks {
                                    for instr in &mut b.instrs {
                                        if let MirInstr::Branch(MirOperand::Reg(r), true_bb, false_bb) = instr {
                                            if let Some(MirOperand::ConstBool(b)) = const_params.get(r) {
                                                *instr = MirInstr::Jump(if *b { *true_bb } else { *false_bb });
                                            }
                                        }
                                    }
                                }
                                
                                // Simple DCE: find reachable blocks
                                let mut reachable = HashSet::new();
                                let mut to_visit = vec![0];
                                while let Some(b) = to_visit.pop() {
                                    if !reachable.insert(b) { continue; }
                                    if b < cloned_func.blocks.len() {
                                        if let Some(last) = cloned_func.blocks[b].instrs.last() {
                                            match last {
                                                MirInstr::Jump(target) => to_visit.push(*target as usize),
                                                MirInstr::Branch(_, t, f) => { to_visit.push(*t as usize); to_visit.push(*f as usize); },
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                                
                                // Remove unreachable blocks (clear instructions)
                                for (i, b) in cloned_func.blocks.iter_mut().enumerate() {
                                    if !reachable.contains(&i) {
                                        b.instrs.clear();
                                    }
                                }
                            }
                            
                            // Run escape analysis on the specialized clone and add it to module_ea
                            let clone_ea = escape::run_escape_analysis(&cloned_func, Some(module_ea));
                            module_ea.insert(clone_name.clone(), clone_ea);
                            
                            new_functions.push(cloned_func);
                            
                            signatures_out.insert(clone_name.clone(), call_sig.clone());
                            worklist.push((clone_name.clone(), Some(call_sig.clone())));
                        }

                        if let Some(new_target) = instantiated.get(&(target.clone(), call_sig)) {
                            *target = new_target.clone();
                        }
                    }
                }
            }
        }

        if func_name == "__bs_script_main" {
            module.main_body = func;
        } else {
            if let Some(f) = module.functions.iter_mut().chain(new_functions.iter_mut()).find(|f| f.name == func_name) {
                *f = func;
            }
        }
    }

    // We must keep the original generic templates because they might be passed dynamically
    // as closures (AllocClosure) which requires a default unspecialized signature.
    module.functions.extend(new_functions);
    signatures_out
}
