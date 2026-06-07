//! Compilation pipeline — ties all phases together.

use std::collections::HashMap;
use std::path::PathBuf;
use inkwell::context::Context;
use inkwell::OptimizationLevel;

use oxc::allocator::Allocator;
use oxc::ast::ast::{Statement, ImportDeclarationSpecifier, ModuleExportName};
use module_graph::{ModuleGraph, ModuleResolver};
use hir::{lower_module_with_imports, BindingId, FuncId, HirModule};
use parser::parse_module;

use rayon::prelude::*;

use diagnostics::CompileResult;
use crate::config::CompileConfig;
use incremental::IncrementalCache;

pub fn run(cfg: &CompileConfig) -> CompileResult<()> {
    tracing::info!("Building module graph from {}", cfg.input.display());
    
    // 1. Build the module graph
    let graph = ModuleGraph::build_from_entry(&cfg.input)?;
    
    // 2. Topologically sort the modules
    let sorted_indices = graph.toposort()?;
    
    // Parallel validation of all module paths in the graph using Rayon
    sorted_indices.par_iter().try_for_each(|&node_idx| {
        let node = &graph.graph[node_idx];
        if !node.path.exists() {
            return Err(diagnostics::CompileError::Lowering {
                message: format!("Module path does not exist: {}", node.path.display()),
            });
        }
        Ok(())
    })?;
    
    let mut compiled_modules = HashMap::<PathBuf, HirModule>::new();
    let resolver = ModuleResolver::new();
    
    let mut next_binding_id: BindingId = 0;
    let mut next_func_id: FuncId = 1; // func 0 is reserved for main
    
    // Setup incremental cache
    let cache_dir = std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".binscript_cache");
    let cache = IncrementalCache::new(cache_dir.clone());
    let current_version = env!("CARGO_PKG_VERSION");
    if !cfg.no_cache {
        let version_file = cache_dir.join("version.txt");
        let version_changed = if version_file.exists() {
            std::fs::read_to_string(&version_file)
                .map(|v| v.trim() != current_version)
                .unwrap_or(true)
        } else {
            true
        };
        if version_changed {
            tracing::info!("Compiler version changed or cache is new. Invalidating cache.");
            cache.invalidate_all();
            let _ = std::fs::write(&version_file, current_version);
        }
    }

    // 3. Compile all modules in topological order
    for &node_idx in &sorted_indices {
        let node = &graph.graph[node_idx];
        let current_path = &node.path;
        let base_dir = current_path.parent().unwrap_or(std::path::Path::new("."));
        
        tracing::info!("Lowering module: {}", current_path.display());
        
        let allocator = Allocator::default();
        let program = parse_module(&allocator, &node.source, current_path.to_str().unwrap_or("module"))?;
        
        let mut import_bindings = HashMap::new();
        let mut import_functions = HashMap::new();
        let mut import_classes = HashMap::new();
        
        // Resolve imports using already compiled modules
        for stmt in &program.body {
            if let Statement::ImportDeclaration(import) = stmt {
                let specifier = import.source.value.to_string();
                let dep_path = resolver.resolve(base_dir, &specifier)?;
                let dep_path = std::fs::canonicalize(&dep_path).unwrap_or(dep_path);
                
                if let Some(dep_module) = compiled_modules.get(&dep_path) {
                    if let Some(specifiers) = &import.specifiers {
                        for spec in specifiers {
                            match spec {
                                ImportDeclarationSpecifier::ImportSpecifier(named) => {
                                    let local = named.local.name.to_string();
                                    let imported = match &named.imported {
                                        ModuleExportName::IdentifierName(id) => id.name.to_string(),
                                        ModuleExportName::IdentifierReference(id) => id.name.to_string(),
                                        ModuleExportName::StringLiteral(s) => s.value.to_string(),
                                    };
                                    
                                    if let Some(&func_id) = dep_module.exports.functions.get(&imported) {
                                        if let Some(f) = dep_module.functions.iter().find(|f| f.id == func_id) {
                                            import_functions.insert(local.clone(), f.name.clone());
                                        }
                                    } else if let Some(class_name) = dep_module.exports.classes.get(&imported) {
                                        import_classes.insert(local.clone(), class_name.clone());
                                    } else if let Some(&bid) = dep_module.exports.named.get(&imported) {
                                        import_bindings.insert(local, bid);
                                    }
                                }
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(default) => {
                                    let local = default.local.name.to_string();
                                    if let Some(&func_id) = dep_module.exports.functions.get("default") {
                                        if let Some(f) = dep_module.functions.iter().find(|f| f.id == func_id) {
                                            import_functions.insert(local.clone(), f.name.clone());
                                        }
                                    } else if let Some(class_name) = dep_module.exports.classes.get("default") {
                                        import_classes.insert(local.clone(), class_name.clone());
                                    } else if let Some(bid) = dep_module.exports.default {
                                        import_bindings.insert(local, bid);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        
        // Cache lookup
        let mut cached_module = None;
        let mut cache_key = None;
        
        if !cfg.no_cache {
            let key = IncrementalCache::compute_key(
                &node.source,
                current_version,
                cfg.opt_level,
                next_binding_id,
                next_func_id,
                &import_bindings,
                &import_functions,
                &import_classes,
            );
            if let Some(module) = cache.lookup(&key) {
                tracing::info!("Cache hit for module: {}", current_path.display());
                cached_module = Some(module);
            } else {
                tracing::info!("Cache miss for module: {}", current_path.display());
                cache_key = Some(key);
            }
        } else {
            tracing::info!("Lowering module: {}", current_path.display());
        }
        
        let mut hir_module = if let Some(module) = cached_module {
            module
        } else {
            let lowered = lower_module_with_imports(
                &program,
                import_bindings,
                import_functions,
                import_classes,
                next_binding_id,
                next_func_id,
            )?;
            if let Some(ref key) = cache_key {
                cache.store(key, &lowered);
            }
            lowered
        };
        
        // Resolve re-exports and export-alls for the current module
        let re_exports_to_resolve = std::mem::take(&mut hir_module.exports.re_exports);
        for re_exp in re_exports_to_resolve {
            let dep_path = resolver.resolve(base_dir, &re_exp.src)?;
            let dep_path = std::fs::canonicalize(&dep_path).unwrap_or(dep_path);
            
            if let Some(dep_module) = compiled_modules.get(&dep_path) {
                if re_exp.local == "default" {
                    if let Some(bid) = dep_module.exports.default {
                        hir_module.exports.named.insert(re_exp.exported, bid);
                    }
                } else {
                    if let Some(&bid) = dep_module.exports.named.get(&re_exp.local) {
                        hir_module.exports.named.insert(re_exp.exported, bid);
                    }
                }
            }
        }
        
        let export_alls_to_resolve = std::mem::take(&mut hir_module.exports.export_alls);
        for src in export_alls_to_resolve {
            let dep_path = resolver.resolve(base_dir, &src)?;
            let dep_path = std::fs::canonicalize(&dep_path).unwrap_or(dep_path);
            
            if let Some(dep_module) = compiled_modules.get(&dep_path) {
                for (name, &bid) in &dep_module.exports.named {
                    if name != "default" {
                        hir_module.exports.named.insert(name.clone(), bid);
                    }
                }
            }
        }
        
        next_binding_id = hir_module.next_binding_id;
        next_func_id = hir_module.next_func_id;
        
        compiled_modules.insert(current_path.clone(), hir_module);
    }
    
    // 4. Merge all HIR modules in topological order
    tracing::info!("Merging modules");
    let mut combined_stmts = Vec::new();
    let mut combined_functions = Vec::new();
    let mut combined_classes = HashMap::new();
    let mut combined_capture_cells = std::collections::HashSet::new();
    
    for &node_idx in &sorted_indices {
        let node = &graph.graph[node_idx];
        if let Some(hir_module) = compiled_modules.remove(&node.path) {
            combined_stmts.extend(hir_module.stmts);
            combined_functions.extend(hir_module.functions);
            combined_classes.extend(hir_module.classes);
            combined_capture_cells.extend(hir_module.capture_cells);
        }
    }
    
    let combined_hir = HirModule {
        stmts: combined_stmts,
        functions: combined_functions,
        classes: combined_classes,
        capture_cells: combined_capture_cells,
        next_binding_id,
        next_func_id,
        exports: hir::ModuleExports::default(),
    };
    
    tracing::debug!(
        "Combined HIR: {} stmts, {} functions",
        combined_hir.stmts.len(),
        combined_hir.functions.len()
    );

    // ── Phase 5: Lower to MIR ──────────────────────────────────────────────
    tracing::info!("Lowering to MIR");
    let mir_module = mir::lower_module(&combined_hir)?;
    tracing::debug!(
        "MIR: {} functions + main ({} blocks)",
        mir_module.functions.len(),
        mir_module.main_body.blocks.len()
    );

    // ── Phase 8: LLVM codegen ──────────────────────────────────────────────
    tracing::info!("Generating LLVM IR");
    let ctx = Context::create();
    let file_name = cfg.input.display().to_string();
    let mut codegen = codegen_llvm::LlvmCodegen::new(&ctx, &file_name);
    codegen.emit_module(&mir_module)?;
    codegen.verify()?;

    if cfg.emit_llvm_ir {
        println!("{}", codegen.print_ir());
        return Ok(());
    }

    // ── Link ───────────────────────────────────────────────────────────────
    let opt = match cfg.opt_level {
        0 => OptimizationLevel::None,
        1 => OptimizationLevel::Less,
        2 => OptimizationLevel::Default,
        _ => OptimizationLevel::Aggressive,
    };

    tracing::info!("Linking → {}", cfg.output.display());
    codegen_llvm::link_to_binary(codegen.get_module(), &cfg.output, opt)?;
    tracing::info!("Done ✓");

    Ok(())
}
