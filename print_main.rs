use ast_to_mir::compile_ast_to_mir;
use compiler_core::parse;
use std::fs;

fn main() {
    let code = fs::read_to_string("tests/zero_cost/context_clone.ts").unwrap();
    let ast = parse::parse_ts(&code).unwrap();
    let mut module = compile_ast_to_mir(&ast);
    let mut module_ea = std::collections::HashMap::new();
    
    // First, run monomorphize which modifies module in place
    let specialized_signatures = ownership_inference::monomorphize::run_monomorphize_pass(&mut module, &mut module_ea);
    
    for block in &module.main_body.blocks {
        for instr in &block.instrs {
            println!("{:?}", instr);
        }
    }
}
