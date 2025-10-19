use crate::ast::*;
use crate::errors::CompileError;
use crate::token::{Token, TokenKind};
use std::collections::HashMap;

#[derive(PartialEq, PartialOrd, Clone, Copy)]
enum Precedence {
    Lowest,
    Equals,      // == !=
    LessGreater, // < > <= >=
    Sum,         // + -
    Product,     // * /
}

lazy_static::lazy_static! {
    static ref PRECEDENCES: HashMap<TokenKind, Precedence> = {
        let mut m = HashMap::new();
        m.insert(TokenKind::Eq, Precedence::Equals);
        m.insert(TokenKind::NotEq, Precedence::Equals);
        m.insert(TokenKind::LAngle, Precedence::LessGreater);
        m.insert(TokenKind::RAngle, Precedence::LessGreater);
        m.insert(TokenKind::LtEq, Precedence::LessGreater);
        m.insert(TokenKind::GtEq, Precedence::LessGreater);
        m.insert(TokenKind::Plus, Precedence::Sum);
        m.insert(TokenKind::Minus, Precedence::Sum);
        m.insert(TokenKind::Star, Precedence::Product);
        m
    };
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse_program(&mut self) -> Result<Program, CompileError> {
        let mut statements = Vec::new();
        while self.current_token().kind != TokenKind::Eof {
            statements.push(self.parse_statement()?);
        }
        Ok(Program { statements })
    }

    fn parse_statement(&mut self) -> Result<Statement, CompileError> {
        let stmt = match self.current_token().kind {
            TokenKind::Use => self.parse_use_statement().map(Statement::Use),
            TokenKind::Struct => self.parse_struct_definition().map(Statement::Struct),
            TokenKind::Component => self.parse_component_definition().map(Statement::Component),
            TokenKind::Fn | TokenKind::Server | TokenKind::Async => self.parse_function_definition().map(Statement::Function),
            TokenKind::Let => self.parse_let_statement().map(Statement::Let),
            TokenKind::Return => self.parse_return_statement().map(Statement::Return),
            TokenKind::If => self.parse_if_statement().map(Statement::If),
            _ => self.parse_expression_statement().map(Statement::Expression),
        }?;
        if self.consume_if_matches(&TokenKind::Semicolon) {}
        Ok(stmt)
    }

    fn parse_use_statement(&mut self) -> Result<UseStatement, CompileError> {
        self.expect_and_consume(&TokenKind::Use)?;
        let mut path = vec![self.parse_identifier()?];
        while self.consume_if_matches(&TokenKind::DoubleColon) {
            if self.current_token().kind == TokenKind::LBrace { break; }
            path.push(self.parse_identifier()?);
        }
        let mut imports = Vec::new();
        if self.consume_if_matches(&TokenKind::LBrace) {
            while self.current_token().kind != TokenKind::RBrace {
                imports.push(self.parse_identifier()?);
                if !self.consume_if_matches(&TokenKind::Comma) { break; }
            }
            self.expect_and_consume(&TokenKind::RBrace)?;
        }
        Ok(UseStatement { path, imports })
    }
    
    fn parse_struct_definition(&mut self) -> Result<StructDefinition, CompileError> {
        self.expect_and_consume(&TokenKind::Struct)?;
        let name = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            let field_name = self.parse_identifier()?;
            self.expect_and_consume(&TokenKind::Colon)?;
            let field_type = self.parse_type_expression()?;
            fields.push((field_name, field_type));
            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::RBrace)?;
        Ok(StructDefinition { name, fields })
    }

    fn parse_component_definition(&mut self) -> Result<ComponentDefinition, CompileError> {
        self.expect_and_consume(&TokenKind::Component)?;
        let name = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::LParen)?;
        let mut parameters = Vec::new();
        while self.current_token().kind != TokenKind::RParen {
            let param_name = self.parse_identifier()?;
            self.expect_and_consume(&TokenKind::Colon)?;
            let param_type = self.parse_type_expression()?;
            parameters.push(FunctionParameter {
                name: param_name,
                type_annotation: param_type,
            });
            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::RParen)?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        // Consume optional 'return' keyword inside component body
        self.consume_if_matches(&TokenKind::Return);

        let body = self.parse_expression(Precedence::Lowest)?;
        self.consume_if_matches(&TokenKind::Semicolon);
        self.expect_and_consume(&TokenKind::RBrace)?;
        Ok(ComponentDefinition {
            name,
            parameters,
            body: Box::new(body),
        })
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDefinition, CompileError> {
        // Check for optional @server or async modifiers
        let is_server = self.consume_if_matches(&TokenKind::Server);
        let is_async = self.consume_if_matches(&TokenKind::Async);

        // Expect fn keyword
        self.expect_and_consume(&TokenKind::Fn)?;

        // Parse function name
        let name = self.parse_identifier()?;

        // Parse parameter list
        self.expect_and_consume(&TokenKind::LParen)?;
        let mut parameters = Vec::new();
        while self.current_token().kind != TokenKind::RParen {
            let param_name = self.parse_identifier()?;
            self.expect_and_consume(&TokenKind::Colon)?;
            let param_type = self.parse_type_expression()?;
            parameters.push(FunctionParameter {
                name: param_name,
                type_annotation: param_type,
            });
            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::RParen)?;

        // Parse optional return type (-> Type)
        let _return_type = if self.consume_if_matches(&TokenKind::Arrow) {
            Some(self.parse_type_expression()?)
        } else {
            None
        };

        // Parse function body (block statement)
        self.expect_and_consume(&TokenKind::LBrace)?;
        let mut statements = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            statements.push(self.parse_statement()?);
        }
        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(FunctionDefinition {
            name,
            parameters,
            is_server,
            is_async,
            body: BlockStatement { statements },
        })
    }

    fn parse_type_expression(&mut self) -> Result<TypeExpression, CompileError> {
        let name = self.parse_identifier()?;
        if self.consume_if_matches(&TokenKind::LAngle) {
            let mut args = Vec::new();
            while self.current_token().kind != TokenKind::RAngle {
                args.push(self.parse_type_expression()?);
                if !self.consume_if_matches(&TokenKind::Comma) { break; }
            }
            self.expect_and_consume(&TokenKind::RAngle)?;
            Ok(TypeExpression::Generic(name, args))
        } else {
            Ok(TypeExpression::Named(name))
        }
    }

    fn parse_let_statement(&mut self) -> Result<LetStatement, CompileError> {
        self.expect_and_consume(&TokenKind::Let)?;
        let name = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::Assign)?;
        let value = self.parse_expression(Precedence::Lowest)?;
        Ok(LetStatement { name, value })
    }

    fn parse_return_statement(&mut self) -> Result<ReturnStatement, CompileError> {
        self.expect_and_consume(&TokenKind::Return)?;
        let value = self.parse_expression(Precedence::Lowest)?;
        Ok(ReturnStatement { value })
    }

    fn parse_if_statement(&mut self) -> Result<IfStatement, CompileError> {
        self.expect_and_consume(&TokenKind::If)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        // Parse then branch
        let mut then_statements = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            then_statements.push(self.parse_statement()?);
        }
        self.expect_and_consume(&TokenKind::RBrace)?;
        let then_branch = BlockStatement { statements: then_statements };

        // Parse optional else branch
        let else_branch = if self.consume_if_matches(&TokenKind::Else) {
            if self.current_token().kind == TokenKind::If {
                // else if
                Some(Box::new(self.parse_if_statement().map(Statement::If)?))
            } else {
                // else block
                self.expect_and_consume(&TokenKind::LBrace)?;
                let mut else_statements = Vec::new();
                while self.current_token().kind != TokenKind::RBrace {
                    else_statements.push(self.parse_statement()?);
                }
                self.expect_and_consume(&TokenKind::RBrace)?;
                Some(Box::new(Statement::Expression(Expression::Identifier(
                    Identifier { value: "__else_block".to_string() }
                )))) // Placeholder, will need better handling
            }
        } else {
            None
        };

        Ok(IfStatement {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_expression_statement(&mut self) -> Result<Expression, CompileError> {
        self.parse_expression(Precedence::Lowest)
    }
    
    fn parse_macro_invocation(&mut self) -> Result<Statement, CompileError> {
        let macro_token = self.current_token().clone();
        self.next_token();
        let mut input_tokens = Vec::new();
        let mut brace_level = 0;
        let mut paren_level = 0;
        loop {
            let token = self.current_token().clone();
            input_tokens.push(token.clone());
            self.next_token();
            match token.kind {
                TokenKind::LParen => paren_level += 1,
                TokenKind::RParen => paren_level -= 1,
                TokenKind::LBrace => brace_level += 1,
                TokenKind::RBrace => brace_level -= 1,
                TokenKind::Eof => return Err(self.error("Unmatched braces in macro invocation")),
                _ => {}
            }
            if paren_level == 0 && brace_level == 0 { break; }
        }
        Ok(Statement::MacroInvocation(MacroInvocation {
            name: Identifier { value: macro_token.lexeme },
            input_tokens,
        }))
    }

    fn parse_expression(&mut self, precedence: Precedence) -> Result<Expression, CompileError> {
        let mut left_expr = self.parse_prefix()?;
        while self.current_token().kind != TokenKind::Semicolon && precedence < self.current_precedence() {
            left_expr = self.parse_infix(left_expr)?;
        }
        Ok(left_expr)
    }

    fn parse_prefix(&mut self) -> Result<Expression, CompileError> {
        let token = self.current_token().clone();
        match &token.kind {
            TokenKind::Identifier => {
                self.next_token();
                let ident_expr = Expression::Identifier(Identifier { value: token.lexeme });
                // Check for function call
                if self.current_token().kind == TokenKind::LParen {
                    return self.parse_function_call(ident_expr);
                }
                Ok(ident_expr)
            },
            TokenKind::Integer(val) => { self.next_token(); Ok(Expression::IntegerLiteral(*val)) },
            TokenKind::Float(val) => { self.next_token(); Ok(Expression::FloatLiteral(val.clone())) },
            TokenKind::String(val) => { self.next_token(); Ok(Expression::StringLiteral(val.clone())) },
            TokenKind::Bool(val) => { self.next_token(); Ok(Expression::BoolLiteral(*val)) },
            TokenKind::LParen => self.parse_lambda_or_grouped(),
            TokenKind::LAngle => self.parse_jsx_element(),
            TokenKind::Pipe => self.parse_lambda_with_pipes(),
            _ => Err(self.error(&format!("No prefix parse function for {:?}", token.kind))),
        }
    }

    fn parse_function_call(&mut self, function: Expression) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::LParen)?;
        let mut arguments = Vec::new();
        while self.current_token().kind != TokenKind::RParen {
            arguments.push(self.parse_expression(Precedence::Lowest)?);
            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::RParen)?;
        Ok(Expression::FunctionCall(FunctionCall {
            function: Box::new(function),
            arguments,
        }))
    }

    fn parse_lambda_or_grouped(&mut self) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::LParen)?;

        // Try to determine if this is a lambda or grouped expression
        // For simplicity, if we see identifier followed by ), => it's a lambda
        if self.current_token().kind == TokenKind::RParen {
            // Empty parameter list for lambda: () =>
            self.expect_and_consume(&TokenKind::RParen)?;
            if self.consume_if_matches(&TokenKind::FatArrow) {
                let body = self.parse_expression(Precedence::Lowest)?;
                return Ok(Expression::Lambda(LambdaExpression {
                    parameters: vec![],
                    body: Box::new(body),
                }));
            }
            // Just empty parens, not lambda
            return Err(self.error("Unexpected empty parentheses"));
        }

        // For now, just treat as grouped expression
        let expr = self.parse_expression(Precedence::Lowest)?;
        self.expect_and_consume(&TokenKind::RParen)?;

        // Check if this is actually a lambda with single param: (x) => body
        if self.consume_if_matches(&TokenKind::FatArrow) {
            if let Expression::Identifier(param) = expr {
                let body = self.parse_expression(Precedence::Lowest)?;
                return Ok(Expression::Lambda(LambdaExpression {
                    parameters: vec![param],
                    body: Box::new(body),
                }));
            }
        }

        Ok(expr)
    }

    fn parse_lambda_with_pipes(&mut self) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::Pipe)?;
        let mut parameters = Vec::new();
        while self.current_token().kind != TokenKind::Pipe {
            parameters.push(self.parse_identifier()?);
            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::Pipe)?;

        // Check for => or just use the next expression
        self.consume_if_matches(&TokenKind::FatArrow);

        let body = self.parse_expression(Precedence::Lowest)?;
        Ok(Expression::Lambda(LambdaExpression {
            parameters,
            body: Box::new(body),
        }))
    }
    
    fn parse_infix(&mut self, left: Expression) -> Result<Expression, CompileError> {
        let operator = self.current_token().clone();
        let precedence = self.current_precedence();
        self.next_token();
        let right = self.parse_expression(precedence)?;
        Ok(Expression::Infix(InfixExpression { left: Box::new(left), operator, right: Box::new(right) }))
    }

    fn parse_jsx_element(&mut self) -> Result<Expression, CompileError> {
        let opening_tag = self.parse_jsx_opening_tag()?;
        let children = if opening_tag.self_closing { vec![] } else { self.parse_jsx_children()? };
        let closing_tag = if opening_tag.self_closing { None } else { Some(self.parse_jsx_closing_tag()?) };
        if !opening_tag.self_closing && opening_tag.name.value != closing_tag.as_ref().unwrap().value {
            return Err(self.error("Mismatched closing tag"));
        }
        Ok(Expression::JsxElement(JsxElement { opening_tag, children, closing_tag }))
    }

    fn parse_jsx_opening_tag(&mut self) -> Result<JsxOpeningTag, CompileError> {
        self.expect_and_consume(&TokenKind::LAngle)?;
        let name = self.parse_identifier()?;
        let mut attributes = vec![];
        while self.current_token().kind != TokenKind::RAngle && self.current_token().kind != TokenKind::Slash {
            attributes.push(self.parse_jsx_attribute()?);
        }
        let self_closing = self.consume_if_matches(&TokenKind::Slash);
        self.expect_and_consume(&TokenKind::RAngle)?;
        Ok(JsxOpeningTag { name, attributes, self_closing })
    }
    
    fn parse_jsx_attribute(&mut self) -> Result<JsxAttribute, CompileError> {
        let name = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::Assign)?;

        // Check if value is wrapped in curly braces for expression interpolation
        if self.consume_if_matches(&TokenKind::LBrace) {
            let value = self.parse_expression(Precedence::Lowest)?;
            self.expect_and_consume(&TokenKind::RBrace)?;
            Ok(JsxAttribute { name, value })
        } else {
            let value = self.parse_expression(Precedence::Lowest)?;
            Ok(JsxAttribute { name, value })
        }
    }

    fn parse_jsx_children(&mut self) -> Result<Vec<JsxChild>, CompileError> {
        let mut children = Vec::new();

        loop {
            // Check if we've reached the closing tag
            if self.current_token().kind == TokenKind::LAngle && self.peek_token().kind == TokenKind::Slash {
                break;
            }

            if self.current_token().kind == TokenKind::Eof {
                return Err(self.error("Unclosed JSX element"));
            }

            // Check for {expr} interpolation
            if self.consume_if_matches(&TokenKind::LBrace) {
                let expr = self.parse_expression(Precedence::Lowest)?;
                self.expect_and_consume(&TokenKind::RBrace)?;
                children.push(JsxChild::Expression(Box::new(expr)));
                continue;
            }

            // Check for nested JSX element (starts with <, but not </)
            if self.current_token().kind == TokenKind::LAngle {
                // Double-check it's not a closing tag
                if self.peek_token().kind == TokenKind::Slash {
                    break; // This is actually the closing tag, exit
                }
                // This is a nested element, parse it
                let child_expr = self.parse_expression(Precedence::Lowest)?;
                if let Expression::JsxElement(el) = child_expr {
                    children.push(JsxChild::Element(Box::new(el)));
                }
                continue;
            }

            // Collect bare text content
            let text = self.collect_jsx_text()?;
            if !text.is_empty() {
                children.push(JsxChild::Text(text));
            }
        }

        Ok(children)
    }

    /// Collect consecutive tokens as JSX text until we hit <, {, or }
    fn collect_jsx_text(&mut self) -> Result<String, CompileError> {
        let mut text = String::new();

        loop {
            let token = self.current_token();

            // Stop at JSX delimiters
            match &token.kind {
                TokenKind::LAngle | TokenKind::LBrace | TokenKind::RBrace | TokenKind::Eof => {
                    break;
                }
                _ => {}
            }

            // Append token content
            if !text.is_empty() && !token.lexeme.is_empty() {
                text.push(' '); // Add spacing between tokens
            }
            text.push_str(&token.lexeme);
            self.next_token();
        }

        Ok(text.trim().to_string())
    }

    fn parse_jsx_closing_tag(&mut self) -> Result<Identifier, CompileError> {
        self.expect_and_consume(&TokenKind::LAngle)?;
        self.expect_and_consume(&TokenKind::Slash)?;
        let name = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::RAngle)?;
        Ok(name)
    }

    fn parse_identifier(&mut self) -> Result<Identifier, CompileError> {
        let token = self.current_token();
        if let TokenKind::Identifier = &token.kind {
            let ident = Identifier { value: token.lexeme.clone() };
            self.next_token();
            Ok(ident)
        } else {
            Err(self.error(&format!("Expected Identifier, found {:?}", token.kind)))
        }
    }

    fn current_token(&self) -> &Token { &self.tokens[self.position] }
    fn peek_token(&self) -> &Token { self.tokens.get(self.position + 1).unwrap_or(self.tokens.last().unwrap()) }
    fn current_precedence(&self) -> Precedence { PRECEDENCES.get(&self.current_token().kind).cloned().unwrap_or(Precedence::Lowest) }
    fn next_token(&mut self) { if self.position < self.tokens.len() - 1 { self.position += 1; } }

    fn expect_and_consume(&mut self, expected: &TokenKind) -> Result<(), CompileError> {
        if &self.current_token().kind == expected {
            self.next_token();
            Ok(())
        } else {
            Err(self.error(&format!("Expected {:?}, found {:?}", expected, self.current_token().kind)))
        }
    }

    fn consume_if_matches(&mut self, kind: &TokenKind) -> bool {
        if &self.current_token().kind == kind {
            self.next_token();
            true
        } else {
            false
        }
    }

    fn error(&self, message: &str) -> CompileError {
        let t = self.current_token();
        CompileError::ParserError { message: message.to_string(), line: t.line, column: t.column }
    }
}