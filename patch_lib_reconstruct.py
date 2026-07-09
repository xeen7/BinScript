with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

# Add pub mod native_sigs; at the top
code = code.replace("pub mod region;", "pub mod region;\npub mod native_sigs;")

# Fix run_escape_analysis call
code = code.replace("let ea = escape::run_escape_analysis(func);", "let ea = escape::run_escape_analysis(func, escape_sigs);")

# Fix classify_registers call
code = code.replace("let mut classes = classify::classify_registers(func, &ag, &ea);", "let mut classes = classify::classify_registers(func, &ag, &ea, return_allocations, param_escapes);")

# Wait, does `run_ownership_inference` receive these arguments?
# Let's check `crates/ownership-inference/src/lib.rs` for `pub fn run_ownership_inference`
