use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum CompileError {
    LexerError(String),
    ParserError { message: String, line: usize, column: usize },
    BorrowError(String),
    Generic(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::LexerError(msg) => write!(f, "Lexer Error: {}", msg),
            CompileError::ParserError { message, line, column } => {
                write!(f, "Parser Error [{}:{}]: {}", line, column, message)
            }
            CompileError::BorrowError(msg) => write!(f, "Borrow Error: {}", msg),
            CompileError::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}
impl std::error::Error for CompileError {}