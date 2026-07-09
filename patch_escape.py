import re

with open('crates/ownership-inference/src/escape.rs', 'r') as f:
    content = f.read()

# Replace analyze_module_escapes signature and body
pattern = r"pub fn analyze_module_escapes\(module: &MirModule\) -> HashMap<String, HashSet<usize>> \{(.*?)\n\}"
replacement = """pub struct ModuleEscapes {
    pub param_escapes: HashMap<String, HashSet<usize>>,
    pub return_allocations: HashSet<String>,
}

pub fn analyze_module_escapes(module: &MirModule) -> ModuleEscapes {
    let mut param_escapes: HashMap<String, HashSet<usize>> = HashMap::new();
    let mut changed = true;

    for func in &module.functions {
        param_escapes.insert(func.name.clone(), HashSet::new());
    }
    param_escapes.insert(module.main_body.name.clone(), HashSet::new());

    while changed {
        changed = false;
        let mut functions_to_analyze = module.functions.iter().collect::<Vec<_>>();
        functions_to_analyze.push(&module.main_body);

        for func in functions_to_analyze {
            let ea = run_escape_analysis(func, &param_escapes);
            let mut new_escapes = HashSet::new();
            for (idx, (reg, _)) in func.params.iter().enumerate() {
                if ea.does_escape(*reg) {
                    new_escapes.insert(idx);
                }
            }
            let old_escapes = param_escapes.get(&func.name).unwrap();
            if new_escapes != *old_escapes {
                param_escapes.insert(func.name.clone(), new_escapes);
                changed = true;
            }
        }
    }

    let mut return_allocations = HashSet::new();
    let mut functions_to_analyze = module.functions.iter().collect::<Vec<_>>();
    functions_to_analyze.push(&module.main_body);

    for func in functions_to_analyze {
        let ea = run_escape_analysis(func, &param_escapes);
        let mut sources = HashMap::new();
        
        for block in &func.blocks {
            for instr in &block.instrs {
                use mir::types::MirInstr::*;
                match instr {
                    Alloc(dest, _) | AllocShared(dest, _) | AllocAcyclic(dest, _) | 
                    AllocSharedAcyclic(dest, _) | AllocOwned(dest, _) | AllocStack(dest, _) | 
                    AllocArena(dest, _, _) | AllocClosure(dest, _, _) => {
                        sources.insert(*dest, *dest);
                    }
                    Move(dest, mir::types::MirOperand::Reg(r)) => {
                        if let Some(&src) = sources.get(r) {
                            sources.insert(*dest, src);
                        }
                    }
                    _ => {}
                }
            }
        }

        let mut has_returns = false;
        let mut all_returns_are_safe_allocs = true;

        for block in &func.blocks {
            for instr in &block.instrs {
                if let mir::types::MirInstr::Return(Some(mir::types::MirOperand::Reg(r))) = instr {
                    has_returns = true;
                    if let Some(&src_alloc) = sources.get(r) {
                        if ea.prevents_owned(src_alloc) {
                            all_returns_are_safe_allocs = false;
                        }
                    } else {
                        all_returns_are_safe_allocs = false;
                    }
                } else if let mir::types::MirInstr::Return(None) = instr {
                    // return void is fine, but doesn't return an allocation
                    all_returns_are_safe_allocs = false;
                } else if let mir::types::MirInstr::Return(Some(_)) = instr {
                    // return primitive 
                    all_returns_are_safe_allocs = false;
                }
            }
        }

        if has_returns && all_returns_are_safe_allocs {
            return_allocations.insert(func.name.clone());
        }
    }

    ModuleEscapes {
        param_escapes,
        return_allocations,
    }
}"""

content = re.sub(pattern, replacement, content, flags=re.DOTALL)

with open('crates/ownership-inference/src/escape.rs', 'w') as f:
    f.write(content)

