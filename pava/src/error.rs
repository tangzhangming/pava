use std::fmt;

#[derive(Debug)]
pub enum CompileError {
    LexerError(String),
    ParserError(String),
    TypeError(String),
    CodegenError(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::LexerError(msg) => write!(f, "Lexer Error: {}", msg),
            CompileError::ParserError(msg) => write!(f, "Parser Error: {}", msg),
            CompileError::TypeError(msg) => write!(f, "Type Error: {}", msg),
            CompileError::CodegenError(msg) => write!(f, "Codegen Error: {}", msg),
        }
    }
}

impl std::error::Error for CompileError {}

pub type CompileResult<T> = Result<T, CompileError>;
