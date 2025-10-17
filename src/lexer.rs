use crate::token::{Token, TokenKind, KEYWORDS};

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    read_position: usize,
    ch: char,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let mut lexer = Self {
            input: input.chars().collect(),
            position: 0,
            read_position: 0,
            ch: '\0',
            line: 1,
            column: 0,
        };
        lexer.read_char();
        lexer
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let start_col = self.column;
        let token = match self.ch {
           ':' => {
                if self.peek() == ':' {
                    self.read_char();
                    return Token::new(TokenKind::DoubleColon, "::".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::Colon, ":".to_string(), self.line, start_col)
                }
           }
            '=' => {
                if self.peek() == '>' {
                    self.read_char();
                    return Token::new(TokenKind::FatArrow, "=>".to_string(), self.line, start_col);
                } else if self.peek() == '=' {
                    self.read_char();
                    return Token::new(TokenKind::Eq, "==".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::Assign, "=".to_string(), self.line, start_col)
                }
            }
            ';' => Token::new(TokenKind::Semicolon, ";".to_string(), self.line, start_col),
            '|' => Token::new(TokenKind::Pipe, "|".to_string(), self.line, start_col), // NEW
            ',' => Token::new(TokenKind::Comma, ",".to_string(), self.line, start_col),
            '.' => Token::new(TokenKind::Dot, ".".to_string(), self.line, start_col),
            '+' => Token::new(TokenKind::Plus, "+".to_string(), self.line, start_col),
            '*' => Token::new(TokenKind::Star, "*".to_string(), self.line, start_col),
            '!' => {
                if self.peek() == '=' {
                    self.read_char();
                    return Token::new(TokenKind::NotEq, "!=".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::Bang, "!".to_string(), self.line, start_col)
                }
            }
            '(' => Token::new(TokenKind::LParen, "(".to_string(), self.line, start_col),
            ')' => Token::new(TokenKind::RParen, ")".to_string(), self.line, start_col),
            '{' => Token::new(TokenKind::LBrace, "{".to_string(), self.line, start_col),
            '}' => Token::new(TokenKind::RBrace, "}".to_string(), self.line, start_col),
            '[' => Token::new(TokenKind::LBracket, "[".to_string(), self.line, start_col),
            ']' => Token::new(TokenKind::RBracket, "]".to_string(), self.line, start_col),
            '<' => {
                if self.peek() == '=' {
                    self.read_char();
                    return Token::new(TokenKind::LtEq, "<=".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::LAngle, "<".to_string(), self.line, start_col)
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.read_char();
                    return Token::new(TokenKind::GtEq, ">=".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::RAngle, ">".to_string(), self.line, start_col)
                }
            }
            '/' => Token::new(TokenKind::Slash, "/".to_string(), self.line, start_col),
            '-' => {
                if self.peek() == '>' {
                    self.read_char();
                    return Token::new(TokenKind::Arrow, "->".to_string(), self.line, start_col);
                } else {
                    Token::new(TokenKind::Minus, "-".to_string(), self.line, start_col)
                }
            }
            '\0' => Token::new(TokenKind::Eof, "".to_string(), self.line, start_col),
            '"' => return self.read_string(),
            _ => {
                if self.ch.is_alphabetic() || self.ch == '_' {
                    return self.read_identifier();
                } else if self.ch.is_digit(10) {
                    return self.read_number();
                } else {
                    Token::new(TokenKind::Illegal(self.ch), self.ch.to_string(), self.line, start_col)
                }
            }
        };
        self.read_char();
        token
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input[self.read_position];
        }
        self.position = self.read_position;
        self.read_position += 1;
        if self.ch == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
    }

    fn peek(&self) -> char {
        if self.read_position >= self.input.len() {
            '\0'
        } else {
            self.input[self.read_position]
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.ch.is_whitespace() {
                self.read_char();
            } else if self.ch == '/' && self.peek() == '/' {
                while self.ch != '\n' && self.ch != '\0' {
                    self.read_char();
                }
            } else {
                break;
            }
        }
    }

    fn read_identifier(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        while self.ch.is_alphanumeric() || self.ch == '_' {
            self.read_char();
        }
        let literal: String = self.input[start_pos..self.position].iter().collect();

        // Check for boolean literals
        let kind = match literal.as_str() {
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => KEYWORDS.get(literal.as_str()).cloned().unwrap_or(TokenKind::Identifier),
        };

        Token::new(kind, literal, self.line, start_col)
    }

    fn read_number(&mut self) -> Token {
        let start_pos = self.position;
        let start_col = self.column;
        let mut is_float = false;

        while self.ch.is_digit(10) {
            self.read_char();
        }

        // Check for decimal point
        if self.ch == '.' && self.peek().is_digit(10) {
            is_float = true;
            self.read_char(); // consume '.'
            while self.ch.is_digit(10) {
                self.read_char();
            }
        }

        let literal: String = self.input[start_pos..self.position].iter().collect();

        if is_float {
            Token::new(TokenKind::Float(literal.clone()), literal, self.line, start_col)
        } else {
            let value = literal.parse().unwrap_or(0);
            Token::new(TokenKind::Integer(value), literal, self.line, start_col)
        }
    }
    
    fn read_string(&mut self) -> Token {
        let start_pos = self.position + 1;
        let start_col = self.column;
        self.read_char(); // Consume opening '"'
        while self.ch != '"' && self.ch != '\0' {
            self.read_char();
        }
        let literal: String = self.input[start_pos..self.position].iter().collect();
        let token = Token::new(TokenKind::String(literal.clone()), literal, self.line, start_col);
        self.read_char(); // Consume closing '"'
        token
    }
}