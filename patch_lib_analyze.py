with open("crates/ownership-inference/src/lib.rs", "r") as f:
    code = f.read()

code = code.replace("pub mod region;", "pub mod region;\npub mod native_sigs;")

code = code.replace("""fn analyze_function(func: &mut MirFunction, class_sizes: &std::collections::HashMap<String, usize>, acyclic_classes: &std::collections::HashSet<String>) {
    let ag = alias_graph::build_alias_graph(func);
    let ea = escape::run_escape_analysis(func);
    let mut classes = classify::classify_registers(func, &ag, &ea);""", """fn analyze_function(func: &mut MirFunction, class_sizes: &std::collections::HashMap<String, usize>, acyclic_classes: &std::collections::HashSet<String>) {
    let ag = alias_graph::build_alias_graph(func);
    
    let mut return_allocations = std::collections::HashSet::new();
    let mut param_escapes = std::collections::HashMap::new();
    
    // We don't have to populate return_allocations / param_escapes perfectly here unless we do IPO.
    let mut escape_sigs = std::collections::HashMap::new();

    let ea = escape::run_escape_analysis(func, &escape_sigs);
    let mut classes = classify::classify_registers(func, &ag, &ea, &return_allocations, &param_escapes);""")

with open("crates/ownership-inference/src/lib.rs", "w") as f:
    f.write(code)
