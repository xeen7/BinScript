use std::path::PathBuf;

/// Compile-time configuration for a single compilation run.
pub struct CompileConfig {
    /// Path to the TypeScript source file.
    pub input: PathBuf,
    /// Path for the output binary.
    pub output: PathBuf,
    /// If true, dump LLVM IR to stdout and do not link.
    pub emit_llvm_ir: bool,
    /// LLVM optimisation level (0–3).
    pub opt_level: u8,
    /// If true, disable incremental caching.
    pub no_cache: bool,
}
