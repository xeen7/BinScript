use ast_to_mir::compile_ast_to_mir;
use compiler_core::parse;
use ownership_inference::monomorphize::run_monomorphize_pass;
use std::fs;

fn main() {
    let code = fs::read_to_string("tests/zero_cost/context_clone.ts").unwrap();
    let ast = parse::parse_ts(&code).unwrap();
    let mut module = compile_ast_to_mir(&ast);
    let mut module_ea = std::collections::HashMap::new();
    
    let _ = run_monomorphize_pass(&mut module, &mut module_ea);
    
    for f in &module.functions {
        println!("FUNCTION: {}", f.name);
    }
}
