use mir::MirModule;
use mir::types::MirInstr;
use petgraph::graph::Graph;
use std::collections::HashMap;

pub struct CallGraph {
    /// Graph of function dependencies. Edge `A -> B` means function A calls function B.
    pub graph: Graph<String, ()>,
    pub node_indices: HashMap<String, petgraph::graph::NodeIndex>,
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_indices: HashMap::new(),
        }
    }
}

pub fn build_call_graph(module: &MirModule) -> CallGraph {
    let mut cg = CallGraph::new();

    // Add all functions as nodes
    for func in module.functions.iter() {
        let idx = cg.graph.add_node(func.name.clone());
        cg.node_indices.insert(func.name.clone(), idx);
    }
    let idx = cg.graph.add_node("__bs_script_main".to_string());
    cg.node_indices.insert("__bs_script_main".to_string(), idx);

    // Helper closure to extract calls from a function
    let mut extract_calls = |func_name: &str, func: &mir::MirFunction| {
        for block in &func.blocks {
            for instr in &block.instrs {
                if let MirInstr::CallDirect(_, target, _) | MirInstr::CallPure(_, target, _) = instr {
                    if let (Some(&src_idx), Some(&dest_idx)) = (cg.node_indices.get(func_name), cg.node_indices.get(target)) {
                        cg.graph.add_edge(src_idx, dest_idx, ());
                    }
                }
            }
        }
    };

    // Extract calls for all functions
    for func in module.functions.iter() {
        extract_calls(&func.name, func);
    }
    extract_calls("__bs_script_main", &module.main_body);

    cg
}
