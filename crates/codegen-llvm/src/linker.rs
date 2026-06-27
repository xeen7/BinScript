//! Object file emission and system linker invocation.
//!
//! Compiles an LLVM `Module` into a native object file, then invokes the
//! system C compiler (or `lld`) to produce a self-contained binary.

use std::path::{Path, PathBuf};
use std::process::Command;

use inkwell::module::Module;
use inkwell::targets::*;
use inkwell::OptimizationLevel;

use diagnostics::{CompileError, CompileResult};

/// Compile the LLVM module to an object file and link it into a native binary.
pub fn link_to_binary(
    module: &Module<'_>,
    output: &Path,
    opt_level: OptimizationLevel,
) -> CompileResult<PathBuf> {
    // Initialise the native target.
    Target::initialize_native(&InitializationConfig::default()).map_err(|e| {
        CompileError::Link {
            message: format!("Failed to init native target: {}", e),
        }
    })?;

    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|e| CompileError::Link {
        message: format!("Bad triple: {}", e.to_string()),
    })?;

    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            opt_level,
            RelocMode::PIC,
            CodeModel::Default,
        )
        .ok_or_else(|| CompileError::Link {
            message: "Could not create target machine".into(),
        })?;

    // Write object file to a temp path next to the output.
    let obj_path = output.with_extension("o");
    machine
        .write_to_file(module, FileType::Object, &obj_path)
        .map_err(|e| CompileError::Link {
            message: format!("write obj: {}", e.to_string()),
        })?;

    // Locate a C compiler for linking.
    let cc = find_cc()?;

    let mut rt_stubs_path = None;

    let build_profile = match opt_level {
        OptimizationLevel::None => "debug",
        _ => "release",
    };

    // 1. Try to find relative to workspace root (CWD), preferring the target profile
    let cwd_paths = [
        PathBuf::from("target").join(build_profile).join("libts_rt_stubs.a"),
        PathBuf::from("lib").join("libts_rt_stubs.a"),
    ];

    for path in &cwd_paths {
        if path.exists() {
            rt_stubs_path = Some(path.to_path_buf());
            break;
        }
    }

    // 2. If not found, try to locate relative to the current compiler executable
    if rt_stubs_path.is_none() {
        let exe_candidates = [
            std::env::current_exe().ok().and_then(|p| p.parent().and_then(|p| p.parent().map(|p| p.join(build_profile).join("libts_rt_stubs.a")))),
            std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.join("libts_rt_stubs.a"))),
        ];
        for candidate_opt in &exe_candidates {
            if let Some(candidate) = candidate_opt {
                if candidate.exists() {
                    rt_stubs_path = Some(candidate.to_path_buf());
                    break;
                }
            }
        }
    }

    let mut cmd = Command::new(&cc);
    cmd.arg(&obj_path);
    if let Some(ref path) = rt_stubs_path {
        cmd.arg("-Wl,--whole-archive");
        cmd.arg(path);
        cmd.arg("-Wl,--no-whole-archive");
    } else {
        // Fallback checks just in case
        let target_release = Path::new("target/release/libts_rt_stubs.a");
        if target_release.exists() {
            cmd.arg(target_release);
        } else {
            let target_debug = Path::new("target/debug/libts_rt_stubs.a");
            if target_debug.exists() {
                cmd.arg(target_debug);
            }
        }
    }

    let status = cmd
        .arg("-o")
        .arg(output)
        .arg("-u")
        .arg("__bs_personality_v0")
        .arg("-u")
        .arg("__bs_rc_flush")
        .arg("-lstdc++") // personality function and C++ runtime
        .arg("-lgcc_s")  // dynamic unwinder (provides _Unwind_RaiseException)
        .arg("-lm") // link libm for math functions
        .arg("-no-pie")
        .arg("-Wl,--gc-sections")   // Garbage collect unused sections
        .arg("-g")                  // Include debug symbols
        .arg("-O3")                 // Optimize linked binary
        .status()
        .map_err(|e| CompileError::Link {
            message: format!("Failed to run linker `{}`: {}", cc, e),
        })?;

    if !status.success() {
        return Err(CompileError::Link {
            message: format!("Linker exited with {}", status),
        });
    }

    // Clean up object file.
    let _ = std::fs::remove_file(&obj_path);

    Ok(output.to_path_buf())
}

/// Find a C compiler to use as a linker driver.
fn find_cc() -> CompileResult<String> {
    // Try well-known compilers in order of preference.
    for name in &["cc", "gcc", "clang"] {
        if which::which(name).is_ok() {
            return Ok(name.to_string());
        }
    }
    Err(CompileError::Link {
        message: "No C compiler found (tried cc, gcc, clang). Install one to link binaries.".into(),
    })
}
