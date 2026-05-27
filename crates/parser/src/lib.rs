use swc_core::common::{
    FileName, Globals, Mark, SourceMap, GLOBALS,
    sync::Lrc,
};
use swc_core::ecma::ast::{EsVersion, Module, Program, Pass};
use swc_core::ecma::parser::lexer::Lexer;
use swc_core::ecma::parser::{Parser, StringInput, Syntax, TsSyntax};
use swc_core::ecma::transforms::base::{fixer, hygiene, resolver};
use swc_core::ecma::transforms::typescript::strip;

use diagnostics::{CompileError, CompileResult};

/// Result of parsing a TypeScript source file.
pub struct ParseResult {
    pub module: Module,
    pub source_map: Lrc<SourceMap>,
}

/// Parse a TypeScript source string and strip all type annotations.
///
/// Runs the full SWC pipeline: parse → resolve → strip TS types → hygiene → fixer.
/// The returned AST is pure ES2022 JavaScript.
pub fn parse_and_strip(src: &str, file_name: &str) -> CompileResult<ParseResult> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(file_name.into()).into(),
        src.to_string(),
    );

    let lexer = Lexer::new(
        Syntax::Typescript(TsSyntax {
            tsx: file_name.ends_with(".tsx"),
            decorators: true,
            ..Default::default()
        }),
        EsVersion::Es2022,
        StringInput::from(&*fm),
        None,
    );

    let mut parser = Parser::new_from(lexer);
    let module = parser.parse_module().map_err(|e| {
        // SWC parse errors must be emitted via the handler, but we capture the message
        CompileError::Parse {
            message: format!("{:?}", e),
        }
    })?;

    let module = GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();

        let mut program = Program::Module(module);

        resolver(unresolved_mark, top_level_mark, true).process(&mut program);
        strip(unresolved_mark, top_level_mark).process(&mut program);
        hygiene::hygiene().process(&mut program);
        fixer::fixer(None).process(&mut program);

        match program {
            Program::Module(m) => m,
            _ => unreachable!(),
        }
    });

    Ok(ParseResult {
        module,
        source_map: cm,
    })
}
