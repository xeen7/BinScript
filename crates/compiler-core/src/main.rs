//! BinScript — TypeScript to native binary compiler.

mod config;
mod pipeline;

use std::path::PathBuf;
use std::process;

use clap::Parser;

#[derive(Parser)]
#[command(name = "binscript", version, about = "Compile TypeScript to native binaries")]
struct Cli {
    /// Input TypeScript file.
    input: PathBuf,

    /// Output binary path (default: input stem).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Print LLVM IR to stdout instead of compiling.
    #[arg(long)]
    emit_llvm_ir: bool,

    /// Optimisation level: 0, 1, 2, or 3.
    #[arg(long, default_value_t = 2)]
    opt_level: u8,

    /// Disable incremental compilation cache.
    #[arg(long)]
    no_cache: bool,

    /// Enable strict memory verification (UAF tracking).
    #[arg(long)]
    verify_memory: bool,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();

    let cfg = config::CompileConfig {
        input: cli.input.clone(),
        output: cli.output.unwrap_or_else(|| {
            let stem = cli.input.file_stem().unwrap_or_default();
            PathBuf::from(stem)
        }),
        emit_llvm_ir: cli.emit_llvm_ir,
        opt_level: cli.opt_level.min(3),
        no_cache: cli.no_cache,
        verify_memory: cli.verify_memory,
    };

    if let Err(e) = pipeline::run(&cfg) {
        eprintln!("error: {}", e);
        process::exit(1);
    }
}
