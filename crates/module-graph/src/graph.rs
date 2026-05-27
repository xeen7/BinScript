use std::collections::{HashMap, VecDeque};
use std::path::{Path, PathBuf};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo::toposort as petgraph_toposort;
use swc_core::ecma::ast::{Module, ModuleDecl, ModuleItem};
use diagnostics::{CompileError, CompileResult};
use parser::parse_and_strip;
use crate::resolve::ModuleResolver;

pub struct ModuleNode {
    pub path: PathBuf,
    pub source: String,
    pub ast: Module,
}

pub struct ModuleGraph {
    pub graph: DiGraph<ModuleNode, ()>,
    pub entry_index: NodeIndex,
    pub path_to_index: HashMap<PathBuf, NodeIndex>,
}

impl ModuleGraph {
    pub fn build_from_entry(entry_path: &Path) -> CompileResult<Self> {
        let resolver = ModuleResolver::new();
        let mut graph = DiGraph::<ModuleNode, ()>::new();
        let mut path_to_index = HashMap::new();
        let mut queue = VecDeque::new();

        let canonical_entry = std::fs::canonicalize(entry_path).map_err(|e| {
            CompileError::Lowering {
                message: format!("Failed to canonicalize entry path '{}': {}", entry_path.display(), e),
            }
        })?;

        let source = std::fs::read_to_string(&canonical_entry).map_err(|e| {
            CompileError::Lowering {
                message: format!("Failed to read entry file '{}': {}", canonical_entry.display(), e),
            }
        })?;

        let parse_res = parse_and_strip(&source, canonical_entry.to_str().unwrap_or("entry"))?;
        
        let entry_node = ModuleNode {
            path: canonical_entry.clone(),
            source,
            ast: parse_res.module,
        };
        
        let entry_index = graph.add_node(entry_node);
        path_to_index.insert(canonical_entry.clone(), entry_index);
        queue.push_back((canonical_entry, entry_index));

        while let Some((current_path, current_index)) = queue.pop_front() {
            let ast = &graph[current_index].ast;
            let deps = get_module_dependencies(ast);
            let base_dir = current_path.parent().unwrap_or(Path::new("."));

            for dep_specifier in deps {
                let resolved_path = resolver.resolve(base_dir, &dep_specifier)?;
                let resolved_path = std::fs::canonicalize(&resolved_path).unwrap_or(resolved_path);

                let dep_index = if let Some(&idx) = path_to_index.get(&resolved_path) {
                    idx
                } else {
                    let source = std::fs::read_to_string(&resolved_path).map_err(|e| {
                        CompileError::Lowering {
                            message: format!("Failed to read imported file '{}': {}", resolved_path.display(), e),
                        }
                    })?;
                    let parse_res = parse_and_strip(&source, resolved_path.to_str().unwrap_or("dep"))?;
                    
                    let node = ModuleNode {
                        path: resolved_path.clone(),
                        source,
                        ast: parse_res.module,
                    };
                    
                    let idx = graph.add_node(node);
                    path_to_index.insert(resolved_path.clone(), idx);
                    queue.push_back((resolved_path, idx));
                    idx
                };

                // Add edge: dep_index -> current_index
                // This means dependency is evaluated before current module.
                graph.add_edge(dep_index, current_index, ());
            }
        }

        Ok(Self {
            graph,
            entry_index,
            path_to_index,
        })
    }

    pub fn toposort(&self) -> CompileResult<Vec<NodeIndex>> {
        match petgraph_toposort(&self.graph, None) {
            Ok(indices) => Ok(indices),
            Err(cycle) => {
                let node_path = &self.graph[cycle.node_id()].path;
                Err(CompileError::Lowering {
                    message: format!(
                        "Circular dependency detected involving module: '{}'",
                        node_path.display()
                    ),
                })
            }
        }
    }
}

fn get_module_dependencies(module: &Module) -> Vec<String> {
    let mut deps = Vec::new();
    for item in &module.body {
        if let ModuleItem::ModuleDecl(decl) = item {
            match decl {
                ModuleDecl::Import(import) => {
                    deps.push(import.src.value.as_wtf8().to_string_lossy().into_owned());
                }
                ModuleDecl::ExportNamed(export) => {
                    if let Some(src) = &export.src {
                        deps.push(src.value.as_wtf8().to_string_lossy().into_owned());
                    }
                }
                ModuleDecl::ExportAll(export) => {
                    deps.push(export.src.value.as_wtf8().to_string_lossy().into_owned());
                }
                _ => {}
            }
        }
    }
    deps
}
