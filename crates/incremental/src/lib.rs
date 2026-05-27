use std::path::PathBuf;
use std::fs;
use std::collections::HashMap;
use sha2::{Sha256, Digest};
use hir::HirModule;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheKey {
    pub hash: [u8; 32],
}

impl CacheKey {
    pub fn to_hex(&self) -> String {
        let mut hex = String::with_capacity(64);
        for byte in &self.hash {
            hex.push_str(&format!("{:02x}", byte));
        }
        hex
    }
}

pub struct IncrementalCache {
    cache_dir: PathBuf,
}

impl IncrementalCache {
    pub fn new(cache_dir: PathBuf) -> Self {
        if !cache_dir.exists() {
            let _ = fs::create_dir_all(&cache_dir);
        }
        Self { cache_dir }
    }

    pub fn lookup(&self, key: &CacheKey) -> Option<HirModule> {
        let path = self.cache_dir.join(format!("{}.bin", key.to_hex()));
        if !path.exists() {
            return None;
        }

        let bytes = fs::read(&path).ok()?;
        bincode::deserialize(&bytes).ok()
    }

    pub fn store(&self, key: &CacheKey, module: &HirModule) {
        let path = self.cache_dir.join(format!("{}.bin", key.to_hex()));
        if let Ok(bytes) = bincode::serialize(module) {
            let _ = fs::write(&path, bytes);
        }
    }

    pub fn invalidate_all(&self) {
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    pub fn compute_key(
        source: &str,
        compiler_version: &str,
        opt_level: u8,
        next_binding_id: u32,
        next_func_id: u32,
        import_bindings: &HashMap<String, u32>,
        import_functions: &HashMap<String, String>,
        import_classes: &HashMap<String, String>,
    ) -> CacheKey {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        hasher.update(compiler_version.as_bytes());
        hasher.update(&[opt_level]);
        hasher.update(&next_binding_id.to_le_bytes());
        hasher.update(&next_func_id.to_le_bytes());

        // Sort import maps to ensure deterministic hashing
        let mut sorted_bindings: Vec<(&String, &u32)> = import_bindings.iter().collect();
        sorted_bindings.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in sorted_bindings {
            hasher.update(k.as_bytes());
            hasher.update(&v.to_le_bytes());
        }

        let mut sorted_functions: Vec<(&String, &String)> = import_functions.iter().collect();
        sorted_functions.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in sorted_functions {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }

        let mut sorted_classes: Vec<(&String, &String)> = import_classes.iter().collect();
        sorted_classes.sort_by(|a, b| a.0.cmp(b.0));
        for (k, v) in sorted_classes {
            hasher.update(k.as_bytes());
            hasher.update(v.as_bytes());
        }

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        CacheKey { hash }
    }
}
