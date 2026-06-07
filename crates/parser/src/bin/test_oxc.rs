use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    let allocator = Allocator::default();
    let src = "function sql(strings, ...values) {}";
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    
    for stmt in ret.program.body {
        if let oxc::ast::ast::Statement::FunctionDeclaration(f) = stmt {
            println!("items length: {}", f.params.items.len());
            println!("has rest: {}", f.params.rest.is_some());
        }
    }
}
