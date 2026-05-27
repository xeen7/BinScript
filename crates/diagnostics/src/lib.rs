use miette::Diagnostic;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum CompileError {
    #[error("Parse error: {message}")]
    #[diagnostic(code(binscript::parse))]
    Parse { message: String },

    #[error("Lowering error: {message}")]
    #[diagnostic(code(binscript::lower))]
    Lowering { message: String },

    #[error("Codegen error: {message}")]
    #[diagnostic(code(binscript::codegen))]
    Codegen { message: String },

    #[error("Link error: {message}")]
    #[diagnostic(code(binscript::link))]
    Link { message: String },

    #[error("IO error: {source}")]
    #[diagnostic(code(binscript::io))]
    Io {
        #[from]
        source: std::io::Error,
    },
}

pub type CompileResult<T> = Result<T, CompileError>;
