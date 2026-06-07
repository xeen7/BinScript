use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::span::SourceType;
use diagnostics::{CompileError, CompileResult};

/// Parse a TypeScript/JavaScript source string.
/// Returns the source text and allocator for use with HIR lowering.
/// Type annotations are preserved in the AST and ignored during lowering.
pub fn parse_module<'a>(
    allocator: &'a Allocator,
    src: &'a str,
    file_name: &str,
) -> CompileResult<oxc::ast::ast::Program<'a>> {
    let source_type = SourceType::from_path(file_name)
        .unwrap_or_default();
    let ret = Parser::new(allocator, src, source_type).parse();
    if !ret.errors.is_empty() {
        return Err(CompileError::Parse {
            message: ret.errors.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n"),
        });
    }
    Ok(ret.program)
}
