

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self { kind, lexeme, line, column }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenKind {
    // Keywords
    Let, Fn, Struct, Component, Extern, Return, Server, Async, Await, Use, True, False, If, Else,

    // Identifiers & Literals
    Identifier,
    Integer(i64),
    Float(String), // Store as string to preserve precision during parsing
    String(String),
    Bool(bool),

    // Symbols & Punctuation
    Assign,      // =
    Semicolon,   // ;
    Colon,       // :
    Comma,       // ,
    Dot,         // .
    Plus,        // +
    Minus,       // -
    Star,        // *
    Bang,        // !
    Pipe,        // | NEW: For closures
    Arrow,       // ->
    FatArrow,    // => NEW: For lambda expressions
    DoubleColon, // :: NEW

    // Comparison operators
    Eq,          // ==
    NotEq,       // !=
    LtEq,        // <=
    GtEq,        // >=

    // Grouping
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }
    LBracket,    // [
    RBracket,    // ]

    // JSX & Comparison
    LAngle,      // <
    RAngle,      // >
    Slash,       // /

    // Meta
    Eof,
    Illegal(char),
}

lazy_static::lazy_static! {
    pub static ref KEYWORDS: std::collections::HashMap<&'static str, TokenKind> = {
        let mut map = std::collections::HashMap::new();
        map.insert("let", TokenKind::Let);
        map.insert("fn", TokenKind::Fn);
        map.insert("struct", TokenKind::Struct);
        map.insert("component", TokenKind::Component);
        map.insert("extern", TokenKind::Extern);
        map.insert("return", TokenKind::Return);
        map.insert("server", TokenKind::Server);
        map.insert("async", TokenKind::Async);
        map.insert("await", TokenKind::Await);
        map.insert("use", TokenKind::Use);
        map.insert("true", TokenKind::True);
        map.insert("false", TokenKind::False);
        map.insert("if", TokenKind::If);
        map.insert("else", TokenKind::Else);
        map
    };
}