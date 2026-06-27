use std::collections::{HashSet, HashMap};
use mir::MirModule;
use hir::HirType;

/// Computes the set of classes that are strictly acyclic.
/// A class is acyclic if its shape dependency graph has no back-edges
/// and does not transitively reach `Any` or natively cyclic built-ins.
pub fn compute_acyclic_classes(module: &MirModule) -> HashSet<String> {
    let mut acyclic = HashSet::new();
    
    // We will build a directed graph where edges point from a Class -> Class it depends on.
    let mut dependencies: HashMap<String, HashSet<String>> = HashMap::new();
    let mut cyclic_roots: HashSet<String> = HashSet::new();

    // Built-in types that are inherently cyclic or opaque.
    // E.g., Array and Closure can hold references to themselves. Map/Set also.
    let builtin_cyclic = vec!["Array", "Closure", "Map", "Set", "Object"];
    
    for (name, class) in &module.classes {
        let mut deps = HashSet::new();
        let mut is_cyclic_root = false;

        // If the class inherits from a super class, it depends on it.
        if let Some(super_name) = &class.super_name {
            deps.insert(super_name.clone());
        }

        for (_, field_type) in &class.fields {
            match field_type {
                HirType::Primitive => {
                    // Primitives cannot form cycles
                }
                HirType::Object(ref_name) => {
                    if builtin_cyclic.contains(&ref_name.as_str()) {
                        is_cyclic_root = true;
                    } else {
                        deps.insert(ref_name.clone());
                    }
                }
                HirType::Any => {
                    // `any` can be anything, including a cycle to this object itself.
                    // Must conservatively mark this class as a cyclic root.
                    is_cyclic_root = true;
                }
            }
        }

        if is_cyclic_root {
            cyclic_roots.insert(name.clone());
        }
        dependencies.insert(name.clone(), deps);
    }

    // Now compute the set of classes that can reach a cyclic_root, or are part of a cycle themselves.
    // We can do this with a simple DFS from each class.
    
    let mut visited: HashSet<String> = HashSet::new();
    let mut processing: HashSet<String> = HashSet::new();
    let mut known_cyclic: HashSet<String> = cyclic_roots.clone();

    // Helper closure to detect cycles via DFS
    fn is_cyclic<'a>(
        node: &'a String,
        deps: &'a HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        processing: &mut HashSet<String>,
        known_cyclic: &mut HashSet<String>,
    ) -> bool {
        if known_cyclic.contains(node) {
            return true;
        }
        if processing.contains(node) {
            // Found a back-edge (cycle)!
            known_cyclic.insert(node.clone());
            return true;
        }
        if visited.contains(node) {
            return false;
        }

        processing.insert(node.clone());
        visited.insert(node.clone());

        let mut node_is_cyclic = false;
        if let Some(edges) = deps.get(node) {
            for target in edges {
                if is_cyclic(target, deps, visited, processing, known_cyclic) {
                    node_is_cyclic = true;
                    // We don't break immediately so we can populate known_cyclic for the whole path
                }
            }
        }

        processing.remove(node);
        
        if node_is_cyclic {
            known_cyclic.insert(node.clone());
            return true;
        }
        
        false
    }

    // Run DFS for all nodes
    for name in module.classes.keys() {
        if !visited.contains(name) {
            is_cyclic(name, &dependencies, &mut visited, &mut processing, &mut known_cyclic);
        }
    }

    // Any class not in `known_cyclic` is strictly acyclic!
    for name in module.classes.keys() {
        if !known_cyclic.contains(name) {
            // Wait, CaptureCell is treated as a struct internally, but it holds `any` value.
            // Oh, we gave it `HirType::Any` in `lower/mod.rs` so it will be correctly marked as cyclic!
            acyclic.insert(name.clone());
        }
    }

    acyclic
}
