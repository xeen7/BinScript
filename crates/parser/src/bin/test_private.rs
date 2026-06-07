use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;

fn main() {
    let allocator = Allocator::default();
    let src = "class A { #x = 1; }";
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    println!("{:#?}", ret.program);
}
