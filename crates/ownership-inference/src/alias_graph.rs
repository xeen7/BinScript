use mir::types::{MirReg, MirInstr, MirFunction, MirOperand};
use petgraph::graphmap::DiGraphMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AliasKind {
    /// A moves to B (B becomes the owner, A is invalidated)
    Move,
    /// B borrows from A (A outlives B, B cannot outlive A)
    Borrow,
    /// B is a mutable borrow of A
    BorrowMut,
    /// A is cloned into B (independent ownership)
    Clone,
    /// A is stored into B (A becomes a child of B)
    Store,
    /// General aliasing (A and B share ownership, usually means RC)
    Alias,
}

pub struct AliasGraph {
    /// Directed graph where nodes are registers, edges are alias relations.
    /// Edge direction is source -> destination.
    pub graph: DiGraphMap<MirReg, AliasKind>,
}

impl AliasGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraphMap::new(),
        }
    }

    pub fn add_edge(&mut self, src: MirReg, dest: MirReg, kind: AliasKind) {
        self.graph.add_edge(src, dest, kind);
    }

    /// Returns true if `reg` has multiple incoming Store/Alias edges
    /// from distinct live scopes — meaning it is aliased.
    pub fn is_aliased(&self, reg: MirReg) -> bool {
        self.graph.neighbors_directed(reg, petgraph::Direction::Incoming)
            .filter(|&src| {
                matches!(
                    self.graph.edge_weight(src, reg),
                    Some(AliasKind::Store | AliasKind::Alias)
                )
            })
            .count() > 1
    }
}

pub fn build_alias_graph(func: &MirFunction) -> AliasGraph {
    let mut ag = AliasGraph::new();

    for block in &func.blocks {
        for instr in &block.instrs {
            match instr {
                MirInstr::Move(dest, src) => {
                    if let MirOperand::Reg(s) = src {
                        ag.add_edge(*s, *dest, AliasKind::Move);
                    }
                }
                MirInstr::StoreField(obj, _, src) |
                MirInstr::StoreSharedField(obj, _, src, _) |
                MirInstr::StoreProp(obj, _, src, _) => {
                    if let MirOperand::Reg(s) = src {
                        ag.add_edge(*s, *obj, AliasKind::Store);
                    }
                }
                MirInstr::LoadField(dest, obj, _) |
                MirInstr::LoadProp(dest, obj, _) => {
                    ag.add_edge(*obj, *dest, AliasKind::Alias);
                }
                MirInstr::AllocClosure(dest, _, captures) => {
                    for cap in captures {
                        if let MirOperand::Reg(s) = cap {
                            ag.add_edge(*s, *dest, AliasKind::Store);
                        }
                    }
                }
                MirInstr::Borrow(dest, src) => {
                    ag.add_edge(*src, *dest, AliasKind::Borrow);
                }
                MirInstr::BorrowMut(dest, src) => {
                    ag.add_edge(*src, *dest, AliasKind::BorrowMut);
                }
                _ => {}
            }
        }
    }

    ag
}
