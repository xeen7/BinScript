use std::path::{Path, PathBuf};
use oxc_resolver::{Resolver, ResolveOptions};
use diagnostics::{CompileError, CompileResult};

pub struct ModuleResolver {
    resolver: Resolver,
}

impl ModuleResolver {
    pub fn new() -> Self {
        let mut options = ResolveOptions::default();
        options.extensions = vec![
            ".ts".to_string(),
            ".tsx".to_string(),
            ".js".to_string(),
            ".jsx".to_string(),
        ];
        Self {
            resolver: Resolver::new(options),
        }
    }

    pub fn resolve(&self, base_dir: &Path, specifier: &str) -> CompileResult<PathBuf> {
        let stripped = specifier.strip_prefix("node:").unwrap_or(specifier);
        if stripped == "fs" || stripped == "path" || stripped == "os" {
            let path = PathBuf::from("rt-stubs/node-compat").join(format!("{}.ts", stripped));
            if path.exists() {
                return Ok(std::fs::canonicalize(path).unwrap());
            }
            if let Ok(exe_path) = std::env::current_exe() {
                if let Some(exe_dir) = exe_path.parent() {
                    let candidate = exe_dir.join("rt-stubs/node-compat").join(format!("{}.ts", stripped));
                    if candidate.exists() {
                        return Ok(std::fs::canonicalize(candidate).unwrap());
                    }
                    if let Some(parent) = exe_dir.parent() {
                        let candidate = parent.join("rt-stubs/node-compat").join(format!("{}.ts", stripped));
                        if candidate.exists() {
                            return Ok(std::fs::canonicalize(candidate).unwrap());
                        }
                    }
                }
            }
            return Err(CompileError::Lowering {
                message: format!("Node compat shim not found for '{}'", specifier),
            });
        }

        match self.resolver.resolve(base_dir, specifier) {
            Ok(resolution) => Ok(resolution.into_path_buf()),
            Err(err) => Err(CompileError::Lowering {
                message: format!(
                    "Failed to resolve module '{}' from '{}': {:?}",
                    specifier,
                    base_dir.display(),
                    err
                ),
            }),
        }
    }
}
