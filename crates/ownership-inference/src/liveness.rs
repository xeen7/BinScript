use mir::types::{MirReg, MirInstr, MirFunction, MirOperand, BlockId};
use std::collections::{HashSet, HashMap};

pub struct LivenessInfo {
    // Maps block ID to a map of instruction index -> registers whose last use is at that index.
    pub last_uses: HashMap<BlockId, HashMap<usize, Vec<MirReg>>>,
    // Maps block ID to registers that must be dropped at the START of the block (due to edge drops)
    pub edge_drops: HashMap<BlockId, Vec<MirReg>>,
}

pub fn split_critical_edges(func: &mut MirFunction) {
    let mut pred_counts: HashMap<BlockId, usize> = HashMap::new();
    for block in &func.blocks {
        pred_counts.insert(block.id, 0);
    }
    for block in &func.blocks {
        if let Some(term) = block.instrs.last() {
            for succ in get_successors(term) {
                *pred_counts.entry(succ).or_insert(0) += 1;
            }
        }
    }

    let mut new_blocks = Vec::new();
    for block in &mut func.blocks {
        let is_multi_succ = if let Some(term) = block.instrs.last() {
            get_successors(term).len() > 1
        } else { false };

        if is_multi_succ {
            let last_idx = block.instrs.len() - 1;
            if let MirInstr::Branch(cond, t, f) = block.instrs[last_idx].clone() {
                let mut new_t = t;
                let mut new_f = f;
                if *pred_counts.get(&t).unwrap_or(&0) > 1 {
                    new_t = func.next_block;
                    func.next_block += 1;
                    new_blocks.push(mir::types::BasicBlock {
                        id: new_t,
                        instrs: vec![MirInstr::Jump(t)],
                        exception_scopes: Vec::new(),
                    });
                }
                if *pred_counts.get(&f).unwrap_or(&0) > 1 {
                    new_f = func.next_block;
                    func.next_block += 1;
                    new_blocks.push(mir::types::BasicBlock {
                        id: new_f,
                        instrs: vec![MirInstr::Jump(f)],
                        exception_scopes: Vec::new(),
                    });
                }
                block.instrs[last_idx] = MirInstr::Branch(cond, new_t, new_f);
            }
        }
    }
    func.blocks.extend(new_blocks);
}

pub fn run_liveness_analysis(func: &mut MirFunction, alias_graph: &crate::alias_graph::AliasGraph) -> LivenessInfo {
    split_critical_edges(func);

    let mut live_in: HashMap<BlockId, HashSet<MirReg>> = HashMap::new();
    let mut live_out: HashMap<BlockId, HashSet<MirReg>> = HashMap::new();
    let mut defs: HashMap<BlockId, HashSet<MirReg>> = HashMap::new();
    let mut uses: HashMap<BlockId, HashSet<MirReg>> = HashMap::new();

    // Compute local defs and uses for each block
    for block in &func.blocks {
        let mut b_defs = HashSet::new();
        let mut b_uses = HashSet::new();

        for instr in &block.instrs {
            let (i_defs, i_uses) = get_defs_uses(instr);
            for u in i_uses {
                if !b_defs.contains(&u) {
                    b_uses.insert(u);
                }
            }
            for d in i_defs {
                b_defs.insert(d);
            }
        }

        defs.insert(block.id, b_defs);
        uses.insert(block.id, b_uses);
        live_in.insert(block.id, HashSet::new());
        live_out.insert(block.id, HashSet::new());
    }

    // Fixed-point iteration for live variables
    let mut changed = true;
    while changed {
        changed = false;

        for block in func.blocks.iter().rev() {
            let mut new_out = HashSet::new();
            if let Some(terminator) = block.instrs.last() {
                let succs = get_successors(terminator);
                for succ in succs {
                    if let Some(succ_in) = live_in.get(&succ) {
                        for &r in succ_in {
                            new_out.insert(r);
                        }
                    }
                }
            }

            let mut new_in = new_out.clone();
            if let Some(b_defs) = defs.get(&block.id) {
                new_in.retain(|r| !b_defs.contains(r));
            }
            if let Some(b_uses) = uses.get(&block.id) {
                for &u in b_uses {
                    new_in.insert(u);
                }
            }

            // Liveness Extension: if a borrowed register is live, its origin MUST be live
            let mut alias_additions = Vec::new();
            for &r in &new_in {
                let origins = alias_graph.get_borrow_origins(r);
                for o in origins {
                    alias_additions.push(o);
                }
            }
            for o in alias_additions {
                new_in.insert(o);
            }

            if live_in.get(&block.id) != Some(&new_in) {
                live_in.insert(block.id, new_in);
                changed = true;
            }
            if live_out.get(&block.id) != Some(&new_out) {
                live_out.insert(block.id, new_out);
                changed = true;
            }
        }
    }

    // Compute last uses
    let mut last_uses: HashMap<BlockId, HashMap<usize, Vec<MirReg>>> = HashMap::new();

    for block in &func.blocks {
        let mut b_last_uses: HashMap<usize, Vec<MirReg>> = HashMap::new();
        let out = live_out.get(&block.id).unwrap();

        let mut currently_live = out.clone();

        for (idx, instr) in block.instrs.iter().enumerate().rev() {
            let (i_defs, i_uses) = get_defs_uses(instr);

            for d in i_defs {
                if !currently_live.contains(&d) {
                    // This variable is defined but never used afterwards in this block.
                    // If it also doesn't live out, then it's completely dead and should be dropped immediately.
                    b_last_uses.entry(idx).or_default().push(d);
                }
                currently_live.remove(&d);
            }

            for u in i_uses {
                if !currently_live.contains(&u) {
                    // This is the last use of 'u' in this block, and it doesn't live out!
                    b_last_uses.entry(idx).or_default().push(u);
                    currently_live.insert(u);
                }
                
                let origins = alias_graph.get_borrow_origins(u);
                for o in origins {
                    if !currently_live.contains(&o) {
                        b_last_uses.entry(idx).or_default().push(o);
                        currently_live.insert(o);
                    }
                }
            }
        }

        last_uses.insert(block.id, b_last_uses);
    }

    let mut edge_drops: HashMap<BlockId, Vec<MirReg>> = HashMap::new();
    for block in &func.blocks {
        if let Some(terminator) = block.instrs.last() {
            let succs = get_successors(terminator);
            for succ in succs {
                let mut drops_on_edge = Vec::new();
                if let Some(b_out) = live_out.get(&block.id) {
                    if let Some(s_in) = live_in.get(&succ) {
                        for &r in b_out {
                            if !s_in.contains(&r) {
                                drops_on_edge.push(r);
                            }
                        }
                    }
                }
                if !drops_on_edge.is_empty() {
                    let entry = edge_drops.entry(succ).or_default();
                    for r in drops_on_edge {
                        if !entry.contains(&r) {
                            entry.push(r);
                        }
                    }
                }
            }
        }
    }

    LivenessInfo { last_uses, edge_drops }
}

fn get_defs_uses(instr: &MirInstr) -> (Vec<MirReg>, Vec<MirReg>) {
    let mut defs = Vec::new();
    let mut uses = Vec::new();

    match instr {
        MirInstr::Alloc(d, _) | MirInstr::AllocShared(d, _) | MirInstr::AllocAcyclic(d, _) | MirInstr::AllocSharedAcyclic(d, _) | MirInstr::AllocOwned(d, _) | MirInstr::AllocStack(d, _) | MirInstr::AllocArena(d, _, _) => defs.push(*d),
        MirInstr::ArenaCreate(_, _) | MirInstr::ArenaDestroy(_) => {},
        MirInstr::LoadField(d, s, _) => { defs.push(*d); uses.push(*s); }
        MirInstr::StoreField(s, _, op) | MirInstr::StoreSharedField(s, _, op, _) => { uses.push(*s); add_op_uses(op, &mut uses); }
        MirInstr::CallVTable(d, s, _, args) | MirInstr::CallClosure(d, s, args) => { defs.push(*d); uses.push(*s); add_args_uses(args, &mut uses); }
        MirInstr::LoadProp(d, s, _) => { defs.push(*d); uses.push(*s); }
        MirInstr::StoreProp(s, _, op, _) => { uses.push(*s); add_op_uses(op, &mut uses); }
        MirInstr::DeleteProp(d, s, op) => { defs.push(*d); add_op_uses(s, &mut uses); add_op_uses(op, &mut uses); }
        MirInstr::AllocClosure(d, _, caps) | MirInstr::AllocSharedClosure(d, _, caps) | MirInstr::AllocOwnedClosure(d, _, caps) => { defs.push(*d); add_args_uses(caps, &mut uses); }
        MirInstr::Suspend(_, op) => add_op_uses(op, &mut uses),
        MirInstr::Resume(d, _) => defs.push(*d),
        MirInstr::TryEnter { .. } | MirInstr::TryExit => {}
        MirInstr::Throw(op) => { add_op_uses(op, &mut uses); }
        MirInstr::Rethrow(r) => { uses.push(*r); }
        MirInstr::LandingPad { exn_reg, .. } => { defs.push(*exn_reg); }
        MirInstr::ExtractException { dest, lp_reg } => { defs.push(*dest); uses.push(*lp_reg); }
        MirInstr::LoadGlobal(d, _) => defs.push(*d),
        MirInstr::StoreGlobal(_, op) => add_op_uses(op, &mut uses),

        MirInstr::Move(d, op) => { defs.push(*d); add_op_uses(op, &mut uses); }
        MirInstr::Add(d, op1, op2) | MirInstr::Sub(d, op1, op2) | MirInstr::Mul(d, op1, op2) |
        MirInstr::Div(d, op1, op2) | MirInstr::Mod(d, op1, op2) | MirInstr::Exp(d, op1, op2) |
        MirInstr::Eq(d, op1, op2) | MirInstr::Ne(d, op1, op2) | MirInstr::StrictEq(d, op1, op2) |
        MirInstr::StrictNe(d, op1, op2) | MirInstr::Lt(d, op1, op2) | MirInstr::Le(d, op1, op2) |
        MirInstr::Gt(d, op1, op2) | MirInstr::Ge(d, op1, op2) | MirInstr::In(d, op1, op2) |
        MirInstr::BitAnd(d, op1, op2) | MirInstr::BitOr(d, op1, op2) | MirInstr::BitXor(d, op1, op2) |
        MirInstr::Shl(d, op1, op2) | MirInstr::Shr(d, op1, op2) | MirInstr::UShr(d, op1, op2) => {
            defs.push(*d); add_op_uses(op1, &mut uses); add_op_uses(op2, &mut uses);
        }
        MirInstr::Neg(d, op) | MirInstr::Plus(d, op) | MirInstr::Not(d, op) | MirInstr::BitNot(d, op) => {
            defs.push(*d); add_op_uses(op, &mut uses);
        }
        MirInstr::CallDirect(d, _, args) | MirInstr::CallPure(d, _, args) | MirInstr::CallBuiltin(d, _, args) => {
            defs.push(*d); add_args_uses(args, &mut uses);
        }
        MirInstr::Branch(op, _, _) => add_op_uses(op, &mut uses),
        MirInstr::Jump(_) => {}
        MirInstr::Return(Some(op)) => add_op_uses(op, &mut uses),
        MirInstr::Return(None) => {}
        MirInstr::RcInc(r) | MirInstr::RcDec(r) | MirInstr::RcIncDeferred(r) | MirInstr::RcDecDeferred(r) | MirInstr::Drop(r) | MirInstr::DropStack(r) | MirInstr::CallDropFnOnly(r) => uses.push(*r),
        MirInstr::FlushRcDelta => {}
        MirInstr::ScopeGuardPush { reg, .. } => uses.push(*reg),
        MirInstr::ScopeGuardCancel { reg, .. } => uses.push(*reg),
        MirInstr::ScopeGuardFlushTo { .. } => {}
        MirInstr::Borrow(d, s) | MirInstr::BorrowMut(d, s) => { defs.push(*d); uses.push(*s); },
        MirInstr::ForceOwnedTag(d) => { defs.push(*d); uses.push(*d); },
    }

    (defs, uses)
}

fn add_op_uses(op: &MirOperand, uses: &mut Vec<MirReg>) {
    if let MirOperand::Reg(r) = op {
        uses.push(*r);
    }
}

fn add_args_uses(args: &[MirOperand], uses: &mut Vec<MirReg>) {
    for arg in args {
        add_op_uses(arg, uses);
    }
}

fn get_successors(instr: &MirInstr) -> Vec<BlockId> {
    match instr {
        MirInstr::Jump(target) => vec![*target],
        MirInstr::Branch(_, target_true, target_false) => vec![*target_true, *target_false],
        _ => vec![],
    }
}
