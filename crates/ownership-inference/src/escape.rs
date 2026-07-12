use mir::types::{MirReg, MirInstr, MirFunction, MirOperand};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EscapeFact {
    Return,
    Throw,
    Store,
    Capture,
    UnknownCall,
    StoreGlobal,
    StoreExternal,
}

#[derive(Clone)]
pub struct EscapeAnalysis {
    pub facts: HashMap<MirReg, HashSet<EscapeFact>>,
    pub param_escapes: Vec<bool>,
    pub param_returns: Vec<bool>,
    pub param_flows: Vec<Vec<usize>>,
    pub returns_fresh_allocation: bool,
}

impl EscapeAnalysis {
    pub fn new(param_count: usize) -> Self {
        Self {
            facts: HashMap::new(),
            param_escapes: Vec::new(),
            param_returns: Vec::new(),
            param_flows: Vec::new(),
            returns_fresh_allocation: false,
        }
    }

    pub fn mark_escape(&mut self, reg: MirReg, fact: EscapeFact) {
        self.facts.entry(reg).or_default().insert(fact);
    }

    pub fn does_escape(&self, reg: MirReg) -> bool {
        match self.facts.get(&reg) {
            None => false,
            Some(facts) => !facts.is_empty(),
        }
    }

    pub fn prevents_owned(&self, reg: MirReg) -> bool {
        match self.facts.get(&reg) {
            None => false,
            Some(facts) => facts.iter().any(|f| matches!(f,
                EscapeFact::Store |
                EscapeFact::Capture |
                EscapeFact::UnknownCall |
                EscapeFact::StoreGlobal |
                EscapeFact::StoreExternal |
                EscapeFact::Throw
            )),
        }
    }

    pub fn prevents_stack(&self, reg: MirReg) -> bool {
        match self.facts.get(&reg) {
            None => false,
            Some(facts) => !facts.is_empty(),
        }
    }
}

pub fn run_escape_analysis(func: &MirFunction, module_ea: Option<&HashMap<String, EscapeAnalysis>>) -> EscapeAnalysis {
    let mut ea = EscapeAnalysis::new(func.params.len());

    let mut allocations = std::collections::HashSet::new();

    // 2. Initial facts based on instructions
    for block in &func.blocks {
        for instr in &block.instrs {
            use mir::types::MirInstr::*;
            match instr {
                Alloc(dest, _) | AllocShared(dest, _) | AllocAcyclic(dest, _) | AllocSharedAcyclic(dest, _) | AllocOwned(dest, _) | AllocStack(dest, _) | AllocArena(dest, _, _) | AllocClosure(dest, _, _) | AllocSharedClosure(dest, _, _) | AllocOwnedClosure(dest, _, _) => {
                    allocations.insert(*dest);
                }
                CallDirect(dest, target, _) => {
                    let mut is_fresh = false;
                    if let Some(sig) = crate::native_sigs::NativeSignature::get(target) {
                        if sig.returns_fresh_allocation {
                            is_fresh = true;
                        }
                    }
                    if let Some(mod_ea) = module_ea {
                        if let Some(target_ea) = mod_ea.get(target) {
                            if target_ea.returns_fresh_allocation {
                                is_fresh = true;
                                if target.contains("closure") {
                                    println!("DEBUG ESCAPE: target {} is fresh in module_ea!", target);
                                }
                            } else {
                                if target.contains("closure") {
                                    println!("DEBUG ESCAPE: target {} is NOT fresh in module_ea", target);
                                }
                            }
                        } else {
                            if target.contains("closure") {
                                println!("DEBUG ESCAPE: target {} is NOT in module_ea", target);
                            }
                        }
                    }
                    if is_fresh {
                        allocations.insert(*dest);
                    }
                }
                CallBuiltin(dest, builtin, _) => {
                    use mir::BuiltinFn::*;
                    match builtin {
                        ArrayNew | ArrayFrom | ArraySlice | ArrayConcat | ArrayMap | ArrayFilter | ArrayFind => {
                            allocations.insert(*dest);
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // 2. Forward propagation for assignments
    let mut alloc_changed = true;
    while alloc_changed {
        alloc_changed = false;
        for block in &func.blocks {
            for instr in &block.instrs {
                use mir::types::MirInstr::*;
                match instr {
                    Move(dest, mir::types::MirOperand::Reg(src)) => {
                        if allocations.contains(src) && allocations.insert(*dest) {
                            alloc_changed = true;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // Reverse dependency graph: if R_from flows into R_to, then graph[R_to] contains R_from.
    // So if R_to escapes, we must also mark R_from as escaping.
    let mut deps: std::collections::HashMap<MirReg, Vec<MirReg>> = std::collections::HashMap::new();

    // Pass 1: Mark direct escapes and build dependency graph
    for block in &func.blocks {
        for instr in &block.instrs {
            match instr {
                MirInstr::Return(Some(MirOperand::Reg(r))) => {
                    ea.mark_escape(*r, EscapeFact::Return);
                }
                MirInstr::Throw(MirOperand::Reg(r)) => {
                    ea.mark_escape(*r, EscapeFact::Throw);
                }
                MirInstr::CallDirect(_, target, args) | MirInstr::CallPure(_, target, args) => {
                    let mut handled_by_ipa = false;
                    let mut is_safe = false;

                    if let Some(mod_ea) = module_ea {
                        if let Some(target_ea) = mod_ea.get(target) {
                            handled_by_ipa = true;
                            let offset = if target_ea.param_escapes.len() > args.len() {
                                target_ea.param_escapes.len() - args.len()
                            } else {
                                0
                            };
        
                            for (idx, arg) in args.iter().enumerate() {
                                if let MirOperand::Reg(r) = arg {
                                    let escapes = target_ea.param_escapes.get(idx + offset).copied().unwrap_or(true);
                                    if target == "__bs_storeIfFlag" {
                                        println!("DEBUG ESCAPE: call to {} arg {} (reg {}) escapes={}", target, idx, r, escapes);
                                    }
                                    if escapes {
                                        ea.mark_escape(*r, EscapeFact::UnknownCall);
                                    }
                                    if target_ea.param_returns.get(idx + offset).copied().unwrap_or(false) {
                                        deps.entry(*r).or_default().push(*r);
                                    }
                                }
                            }
                            
                            for (src_idx, flows_to) in target_ea.param_flows.iter().enumerate() {
                                if src_idx >= offset {
                                    for &dest_idx in flows_to {
                                        if dest_idx >= offset {
                                            if let (Some(MirOperand::Reg(src)), Some(MirOperand::Reg(dest))) = (args.get(src_idx - offset), args.get(dest_idx - offset)) {
                                                deps.entry(*dest).or_default().push(*src);
                                                if !allocations.contains(dest) {
                                                    ea.mark_escape(*src, EscapeFact::StoreExternal);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            if target == "__bs_storeIfFlag" {
                                println!("DEBUG ESCAPE: call to {} NOT IN module_ea", target);
                            }
                        }
                    }

                    if !handled_by_ipa {
                        let sig_opt = crate::native_sigs::NativeSignature::get(target);
                        if let Some(sig) = &sig_opt {
                            if sig.is_safe_stub {
                                is_safe = true;
                                if let Some((src_idx, dest_idx)) = sig.argument_flow {
                                    if let (Some(MirOperand::Reg(dest)), Some(MirOperand::Reg(src))) = (args.get(dest_idx), args.get(src_idx)) {
                                        deps.entry(*dest).or_default().push(*src);
                                        if !allocations.contains(dest) {
                                            ea.mark_escape(*src, EscapeFact::StoreExternal);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !is_safe && !handled_by_ipa {
                        // Constructor 'this' doesn't escape inherently, but other args do
                        let is_constructor = target.ends_with("_constructor") || target.contains("_constructor_sig_");
                        for (idx, arg) in args.iter().enumerate() {
                            if is_constructor && idx == 0 {
                                continue;
                            }
                            if let MirOperand::Reg(r) = arg {
                                ea.mark_escape(*r, EscapeFact::UnknownCall);
                            }
                        }
                    }
                }
                MirInstr::CallBuiltin(_, builtin, args) => {
                    // builtins are assumed to not escape arguments unless specifically known
                    use mir::BuiltinFn;
                    match builtin {
                        BuiltinFn::ArrayPush => {
                            if let MirOperand::Reg(arr) = args[0] {
                                if let MirOperand::Reg(val) = args[1] {
                                    deps.entry(arr).or_default().push(val);
                                    if !allocations.contains(&arr) {
                                        ea.mark_escape(val, EscapeFact::StoreExternal);
                                    }
                                }
                            }
                        }
                        BuiltinFn::ArraySet => {
                            if args.len() >= 3 {
                                if let MirOperand::Reg(arr) = args[0] {
                                    if let MirOperand::Reg(val) = args[2] {
                                        deps.entry(arr).or_default().push(val);
                                        if !allocations.contains(&arr) {
                                            ea.mark_escape(val, EscapeFact::StoreExternal);
                                        }
                                    }
                                }
                            }
                        }
                        // ArrayPop, ArrayLength, ConsoleLog, ArrayMap, ArrayFilter etc. are safe and purely synchronous
                        _ => {}
                    }
                }
                MirInstr::StoreGlobal(_, _) => {
                    // This escapes
                }
                MirInstr::StoreProp(obj, _, MirOperand::Reg(val), _) |
                MirInstr::StoreSharedField(obj, _, MirOperand::Reg(val), _) |
                MirInstr::StoreField(obj, _, MirOperand::Reg(val)) => {
                    deps.entry(*obj).or_default().push(*val);
                    if !allocations.contains(obj) {
                        ea.mark_escape(*val, EscapeFact::StoreExternal);
                    }
                }
                MirInstr::AllocClosure(_, _, captures) | MirInstr::AllocSharedClosure(_, _, captures) | MirInstr::AllocOwnedClosure(_, _, captures) => {
                    for cap in captures {
                        if let MirOperand::Reg(r) = cap {
                            ea.mark_escape(*r, EscapeFact::Capture);
                        }
                    }
                }
                MirInstr::Move(dest, MirOperand::Reg(src)) => {
                    deps.entry(*dest).or_default().push(*src);
                }
                MirInstr::LoadField(dest, obj, _) | MirInstr::LoadProp(dest, obj, _) => {
                    // Not escaping
                }
                _ => {}
            }
        }
    }

    // Determine initial all_returns_fresh based purely on allocations
    let mut all_returns_fresh = true;
    let mut has_returns = false;
    for block in &func.blocks {
        for instr in &block.instrs {
            if let mir::types::MirInstr::Return(Some(mir::types::MirOperand::Reg(r))) = instr {
                has_returns = true;
                if !allocations.contains(r) {
                    all_returns_fresh = false;
                }
            }
        }
    }

    let mut changed = true;
    while changed {
        changed = false;

        if all_returns_fresh {
            for block in &func.blocks {
                for instr in &block.instrs {
                    if let mir::types::MirInstr::Return(Some(mir::types::MirOperand::Reg(r))) = instr {
                        if ea.prevents_owned(*r) {
                            all_returns_fresh = false;
                        }
                    }
                }
            }
        }

        if has_returns && !all_returns_fresh {
            for block in &func.blocks {
                for instr in &block.instrs {
                    if let mir::types::MirInstr::Return(Some(mir::types::MirOperand::Reg(r))) = instr {
                        let facts = ea.facts.get(r).cloned().unwrap_or_default();
                        if !facts.contains(&EscapeFact::StoreExternal) {
                            ea.mark_escape(*r, EscapeFact::StoreExternal);
                            changed = true;
                        }
                    }
                }
            }
        }

        for (to, froms) in &deps {
            if ea.does_escape(*to) {
                let to_facts = ea.facts.get(to).unwrap().clone();
                for from in froms {
                    let from_facts = ea.facts.entry(*from).or_default();
                    for fact in &to_facts {
                        if from_facts.insert(fact.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    // Populate param_escapes based on final results
    for (idx, (reg, _)) in func.params.iter().enumerate() {
        ea.param_escapes.push(ea.does_escape(*reg));
        
        let returns = match ea.facts.get(reg) {
            Some(facts) => facts.contains(&EscapeFact::Return),
            None => false,
        };
        ea.param_returns.push(returns);
    }

    // Populate param_flows
    for (src_idx, (src_reg, _)) in func.params.iter().enumerate() {
        let mut flows_to = Vec::new();
        for (dest_idx, (dest_reg, _)) in func.params.iter().enumerate() {
            if src_idx != dest_idx {
                // BFS to see if src_reg flows into dest_reg.
                // deps maps dest -> [srcs], so we start at dest_reg and search for src_reg.
                let mut visited = std::collections::HashSet::new();
                let mut queue = std::collections::VecDeque::new();
                queue.push_back(*dest_reg);
                
                let mut reachable = false;
                while let Some(curr) = queue.pop_front() {
                    if curr == *src_reg {
                        reachable = true;
                        break;
                    }
                    if !visited.insert(curr) {
                        continue;
                    }
                    if let Some(srcs) = deps.get(&curr) {
                        for &src in srcs {
                            queue.push_back(src);
                        }
                    }
                }
                if reachable {
                    flows_to.push(dest_idx);
                }
            }
        }
        ea.param_flows.push(flows_to);
    }

    ea.returns_fresh_allocation = has_returns && all_returns_fresh;
    println!("DEBUG: Function {} has returns_fresh_allocation={}", func.name, ea.returns_fresh_allocation);

    ea
}
