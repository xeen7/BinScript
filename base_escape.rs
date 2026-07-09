use mir::types::{MirReg, MirInstr, MirFunction, MirOperand};
use std::collections::{HashSet, HashMap};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EscapeFact {
    Return,
    Store,
    Capture,
    UnknownCall,
    StoreGlobal,
}

pub struct EscapeAnalysis {
    pub facts: HashMap<MirReg, HashSet<EscapeFact>>,
}

impl EscapeAnalysis {
    pub fn new() -> Self {
        Self {
            facts: HashMap::new(),
        }
    }

    pub fn mark_escape(&mut self, reg: MirReg, fact: EscapeFact) {
        self.facts.entry(reg).or_default().insert(fact);
    }

    pub fn does_escape(&self, reg: MirReg) -> bool {
        self.facts.contains_key(&reg) && !self.facts[&reg].is_empty()
    }
    
    pub fn prevents_owned(&self, reg: MirReg) -> bool {
        match self.facts.get(&reg) {
            None => false,
            Some(facts) => facts.iter().any(|f| matches!(f,
                EscapeFact::Store |
                EscapeFact::Capture |
                EscapeFact::UnknownCall |
                EscapeFact::StoreGlobal
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

pub fn run_escape_analysis(func: &MirFunction) -> EscapeAnalysis {
    let mut ea = EscapeAnalysis::new();
    
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
                    ea.mark_escape(*r, EscapeFact::Return);
                }
                MirInstr::CallDirect(_, target, args) => {
                    // Constructor 'this' doesn't escape inherently, but other args do
                    let is_constructor = target.ends_with("_constructor");
                    for (idx, arg) in args.iter().enumerate() {
                        if is_constructor && idx == 0 {
                            continue;
                        }
                        if let MirOperand::Reg(r) = arg {
                            ea.mark_escape(*r, EscapeFact::UnknownCall);
                        }
                    }
                }
                MirInstr::CallBuiltin(_, _, args) | MirInstr::CallVTable(_, _, _, args) | MirInstr::CallClosure(_, _, args) => {
                    for arg in args {
                        if let MirOperand::Reg(r) = arg {
                            ea.mark_escape(*r, EscapeFact::UnknownCall);
                        }
                    }
                }
                MirInstr::AllocClosure(_, _, captures) => {
                    for cap in captures {
                        if let MirOperand::Reg(r) = cap {
                            ea.mark_escape(*r, EscapeFact::Capture);
                        }
                    }
                }
                MirInstr::StoreProp(_, _, MirOperand::Reg(r), _) => {
                    ea.mark_escape(*r, EscapeFact::Store);
                }
                MirInstr::StoreField(_, _, MirOperand::Reg(r)) => {
                    ea.mark_escape(*r, EscapeFact::Store);
                }
                MirInstr::StoreSharedField(_, _, MirOperand::Reg(r), _) => {
                    ea.mark_escape(*r, EscapeFact::Store);
                }
                MirInstr::StoreGlobal(_, MirOperand::Reg(r)) => {
                    ea.mark_escape(*r, EscapeFact::StoreGlobal);
                }
                // Dependencies
                MirInstr::Move(dest, MirOperand::Reg(src)) => {
                    deps.entry(*dest).or_default().push(*src);
                }
                MirInstr::LoadField(dest, obj_reg, _) | MirInstr::LoadProp(dest, obj_reg, _) => {
                    deps.entry(*dest).or_default().push(*obj_reg);
                }
                MirInstr::Borrow(dest, src) | MirInstr::BorrowMut(dest, src) => {
                    deps.entry(*dest).or_default().push(*src);
                }
                _ => {}
            }
        }
    }

    // Pass 2: Worklist propagation
    let mut worklist: Vec<MirReg> = ea.facts.keys().copied().collect();
    
    while let Some(reg) = worklist.pop() {
        if let Some(sources) = deps.get(&reg) {
            let facts_to_propagate = ea.facts.get(&reg).cloned().unwrap_or_default();
            for &src in sources {
                let src_facts = ea.facts.entry(src).or_default();
                let mut changed = false;
                for fact in &facts_to_propagate {
                    if src_facts.insert(fact.clone()) {
                        changed = true;
                    }
                }
                if changed {
                    worklist.push(src);
                }
            }
        }
    }

    ea
}
