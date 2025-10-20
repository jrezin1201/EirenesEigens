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
            TokenKind::Enum => self.parse_enum_definition().map(Statement::Enum),
            TokenKind::Impl => self.parse_impl_block().map(Statement::ImplBlock),
            TokenKind::Trait => self.parse_trait_definition().map(Statement::Trait),
            TokenKind::Component => self.parse_component_definition().map(Statement::Component),
            TokenKind::At => {
                // Check if this is @client component or @server/@client function
                let peek = self.peek_token().kind.clone();
                if peek == TokenKind::Client && self.position + 2 < self.tokens.len() {
                    let peek2 = self.tokens[self.position + 2].kind.clone();
                    if peek2 == TokenKind::Component {
                        self.parse_component_definition().map(Statement::Component)
                    } else {
                        self.parse_function_definition().map(Statement::Function)
                    }
                } else {
                    self.parse_function_definition().map(Statement::Function)
                }
            }
            TokenKind::Fn | TokenKind::Server | TokenKind::Client | TokenKind::Async => self.parse_function_definition().map(Statement::Function),
            TokenKind::Let => self.parse_let_statement().map(Statement::Let),
            TokenKind::Return => self.parse_return_statement().map(Statement::Return),
            TokenKind::If => self.parse_if_statement().map(Statement::If),
            TokenKind::While => self.parse_while_statement().map(Statement::While),
            TokenKind::For => {
                // Look ahead to distinguish between for-in and C-style for loop
                // for item in collection { } vs for (init; cond; update) { }
                if self.peek_token().kind == TokenKind::LParen {
                    self.parse_for_statement().map(Statement::For)
                } else {
                    self.parse_for_in_statement().map(Statement::ForIn)
                }
            },
            TokenKind::Identifier => {
                // Check if this is an assignment
                if self.peek_token().kind == TokenKind::Assign {
                    self.parse_assignment_statement().map(Statement::Assignment)
                } else {
                    self.parse_expression_statement().map(Statement::Expression)
                }
            }
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
    
    fn parse_type_params(&mut self) -> Result<Vec<Identifier>, CompileError> {
        if !self.consume_if_matches(&TokenKind::LAngle) {
            return Ok(Vec::new());
        }

        let mut type_params = Vec::new();
        while self.current_token().kind != TokenKind::RAngle {
            type_params.push(self.parse_identifier()?);
            if !self.consume_if_matches(&TokenKind::Comma) {
                break;
            }
        }
        self.expect_and_consume(&TokenKind::RAngle)?;
        Ok(type_params)
    }

    fn parse_struct_definition(&mut self) -> Result<StructDefinition, CompileError> {
        self.expect_and_consume(&TokenKind::Struct)?;
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
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
        Ok(StructDefinition { name, lifetime_params: Vec::new(), type_params, fields })
    }

    fn parse_enum_definition(&mut self) -> Result<EnumDefinition, CompileError> {
        self.expect_and_consume(&TokenKind::Enum)?;
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        let mut variants = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            let variant_name = self.parse_identifier()?;

            // Check if this variant has associated data
            let fields = if self.consume_if_matches(&TokenKind::LBrace) {
                // Struct-style variant: Name { field1: Type, field2: Type }
                let mut variant_fields = Vec::new();
                while self.current_token().kind != TokenKind::RBrace {
                    let field_name = self.parse_identifier()?;
                    self.expect_and_consume(&TokenKind::Colon)?;
                    let field_type = self.parse_type_expression()?;
                    variant_fields.push((field_name, field_type));
                    if !self.consume_if_matches(&TokenKind::Comma) { break; }
                }
                self.expect_and_consume(&TokenKind::RBrace)?;
                Some(variant_fields)
            } else if self.consume_if_matches(&TokenKind::LParen) {
                // Tuple-style variant: Name(Type1, Type2)
                let mut variant_fields = Vec::new();
                let mut index = 0;
                while self.current_token().kind != TokenKind::RParen {
                    let field_type = self.parse_type_expression()?;
                    // For tuple variants, use numeric field names
                    variant_fields.push((
                        Identifier { value: index.to_string() },
                        field_type,
                    ));
                    index += 1;
                    if !self.consume_if_matches(&TokenKind::Comma) { break; }
                }
                self.expect_and_consume(&TokenKind::RParen)?;
                Some(variant_fields)
            } else {
                // Simple variant with no data: Name
                None
            };

            variants.push(EnumVariant {
                name: variant_name,
                fields,
            });

            if !self.consume_if_matches(&TokenKind::Comma) { break; }
        }
        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(EnumDefinition { name, lifetime_params: Vec::new(), type_params, variants })
    }

    fn parse_impl_block(&mut self) -> Result<ImplBlock, CompileError> {
        // impl<T> TypeName { methods... } or impl<T> Trait for TypeName { methods... }
        self.expect_and_consume(&TokenKind::Impl)?;

        // Parse optional type parameters
        let type_params = self.parse_type_params()?;

        let first_name = self.parse_identifier()?;

        // Skip optional type arguments on the first name (e.g., Box<T>)
        // We store type_params separately, so we can ignore these for now
        if self.current_token().kind == TokenKind::LAngle {
            self.parse_type_params()?; // Consume and ignore type arguments
        }

        // Check if this is "impl Trait for Type" or just "impl Type"
        let (trait_name, type_name) = if self.consume_if_matches(&TokenKind::For) {
            // This is a trait implementation
            let type_name = self.parse_identifier()?;

            // Skip optional type arguments on the type name
            if self.current_token().kind == TokenKind::LAngle {
                self.parse_type_params()?; // Consume and ignore type arguments
            }

            (Some(first_name), type_name)
        } else {
            // This is an inherent implementation
            (None, first_name)
        };

        self.expect_and_consume(&TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            // Parse method: fn method_name(...) -> ReturnType { body }
            self.expect_and_consume(&TokenKind::Fn)?;
            let method_name = self.parse_identifier()?;

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
                if !self.consume_if_matches(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect_and_consume(&TokenKind::RParen)?;

            // Parse optional return type (-> Type)
            let return_type = if self.consume_if_matches(&TokenKind::Arrow) {
                Some(self.parse_type_expression()?)
            } else {
                None
            };

            // Parse method body (block statement)
            self.expect_and_consume(&TokenKind::LBrace)?;
            let mut statements = Vec::new();
            while self.current_token().kind != TokenKind::RBrace {
                statements.push(self.parse_statement()?);
            }
            self.expect_and_consume(&TokenKind::RBrace)?;

            methods.push(ImplMethod {
                name: method_name,
                parameters,
                return_type,
                body: BlockStatement { statements },
            });
        }

        self.expect_and_consume(&TokenKind::RBrace)?;
        Ok(ImplBlock { trait_name, lifetime_params: Vec::new(), type_params, type_name, methods })
    }

    fn parse_trait_definition(&mut self) -> Result<TraitDefinition, CompileError> {
        // trait TraitName<T> { method signatures... }
        self.expect_and_consume(&TokenKind::Trait)?;
        let name = self.parse_identifier()?;
        let type_params = self.parse_type_params()?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        let mut methods = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            // Parse method signature: fn method_name(...) -> ReturnType;
            self.expect_and_consume(&TokenKind::Fn)?;
            let method_name = self.parse_identifier()?;

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
                if !self.consume_if_matches(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect_and_consume(&TokenKind::RParen)?;

            // Parse optional return type (-> Type)
            let return_type = if self.consume_if_matches(&TokenKind::Arrow) {
                Some(self.parse_type_expression()?)
            } else {
                None
            };

            // Consume semicolon (trait methods are signatures only)
            self.consume_if_matches(&TokenKind::Semicolon);

            methods.push(TraitMethod {
                name: method_name,
                parameters,
                return_type,
            });
        }

        self.expect_and_consume(&TokenKind::RBrace)?;
        Ok(TraitDefinition { name, lifetime_params: Vec::new(), type_params, methods })
    }

    fn parse_component_definition(&mut self) -> Result<ComponentDefinition, CompileError> {
        // Check for optional @client annotation (components are client-only by default)
        let has_at = self.consume_if_matches(&TokenKind::At);
        let is_client = if has_at {
            self.consume_if_matches(&TokenKind::Client)
        } else {
            true  // Components are client-side by default
        };

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
            is_client,
            body: Box::new(body),
        })
    }

    fn parse_function_definition(&mut self) -> Result<FunctionDefinition, CompileError> {
        // Check for optional @ symbol (for @server or @client)
        let has_at = self.consume_if_matches(&TokenKind::At);

        // Check for server/client annotations (with or without @)
        let is_server = if has_at {
            self.consume_if_matches(&TokenKind::Server)
        } else {
            self.consume_if_matches(&TokenKind::Server)
        };

        let is_client = if has_at && !is_server {
            self.consume_if_matches(&TokenKind::Client)
        } else if !has_at {
            self.consume_if_matches(&TokenKind::Client)
        } else {
            false
        };

        let is_async = self.consume_if_matches(&TokenKind::Async);

        // Expect fn keyword
        self.expect_and_consume(&TokenKind::Fn)?;

        // Parse function name
        let name = self.parse_identifier()?;

        // Parse optional type parameters
        let type_params = self.parse_type_params()?;

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
            lifetime_params: Vec::new(),
            type_params,
            parameters,
            is_server,
            is_client,
            is_async,
            body: BlockStatement { statements },
        })
    }

    fn parse_type_expression(&mut self) -> Result<TypeExpression, CompileError> {
        // Check if this is a slice type [T]
        if self.consume_if_matches(&TokenKind::LBracket) {
            let inner_type = self.parse_type_expression()?;
            self.expect_and_consume(&TokenKind::RBracket)?;
            return Ok(TypeExpression::Slice(Box::new(inner_type)));
        }

        // Check if this is a reference type (&T or &mut T)
        if self.consume_if_matches(&TokenKind::Ampersand) {
            // Check for &mut T (mutable reference)
            if self.consume_if_matches(&TokenKind::Mut) {
                let inner_type = self.parse_type_expression()?;
                return Ok(TypeExpression::MutableReference(Box::new(inner_type)));
            }
            // Otherwise it's &T (immutable reference)
            let inner_type = self.parse_type_expression()?;
            return Ok(TypeExpression::Reference(Box::new(inner_type)));
        }

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

    fn parse_while_statement(&mut self) -> Result<WhileStatement, CompileError> {
        self.expect_and_consume(&TokenKind::While)?;
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        // Parse loop body
        let mut body_statements = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            body_statements.push(self.parse_statement()?);
        }
        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(WhileStatement {
            condition,
            body: BlockStatement { statements: body_statements },
        })
    }

    fn parse_for_statement(&mut self) -> Result<ForStatement, CompileError> {
        self.expect_and_consume(&TokenKind::For)?;
        self.expect_and_consume(&TokenKind::LParen)?;

        // Parse init (optional)
        let init = if self.current_token().kind == TokenKind::Semicolon {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };

        // Consume semicolon after init (if there was one, or standalone)
        if !self.consume_if_matches(&TokenKind::Semicolon) {
            // parse_statement might have consumed it already
        }

        // Parse condition
        let condition = self.parse_expression(Precedence::Lowest)?;
        self.expect_and_consume(&TokenKind::Semicolon)?;

        // Parse update (optional)
        let update = if self.current_token().kind == TokenKind::RParen {
            None
        } else {
            Some(Box::new(self.parse_statement()?))
        };

        self.expect_and_consume(&TokenKind::RParen)?;
        self.expect_and_consume(&TokenKind::LBrace)?;

        // Parse loop body
        let mut body_statements = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            body_statements.push(self.parse_statement()?);
        }
        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(ForStatement {
            init,
            condition,
            update,
            body: BlockStatement { statements: body_statements },
        })
    }

    fn parse_for_in_statement(&mut self) -> Result<ForInStatement, CompileError> {
        // Parse: for item in collection { body }
        self.expect_and_consume(&TokenKind::For)?;

        // Parse loop variable
        let variable = self.parse_identifier()?;

        // Expect 'in' keyword
        self.expect_and_consume(&TokenKind::In)?;

        // Parse iterator expression
        let iterator = self.parse_expression(Precedence::Lowest)?;

        // Parse loop body
        self.expect_and_consume(&TokenKind::LBrace)?;
        let mut body_statements = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            body_statements.push(self.parse_statement()?);
        }
        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(ForInStatement {
            variable,
            iterator,
            body: BlockStatement { statements: body_statements },
        })
    }

    fn parse_assignment_statement(&mut self) -> Result<AssignmentStatement, CompileError> {
        let target = self.parse_identifier()?;
        self.expect_and_consume(&TokenKind::Assign)?;
        let value = self.parse_expression(Precedence::Lowest)?;
        Ok(AssignmentStatement { target, value })
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
                let ident = Identifier { value: token.lexeme };

                // Check if this is a struct literal (Identifier { field: value, ... })
                if self.current_token().kind == TokenKind::LBrace {
                    return self.parse_struct_literal(ident);
                }

                let mut expr = Expression::Identifier(ident);

                // Check for postfix operations (function call, field access, array indexing, or try operator)
                loop {
                    match self.current_token().kind {
                        TokenKind::LParen => {
                            expr = self.parse_function_call(expr)?;
                        }
                        TokenKind::Dot => {
                            self.next_token(); // consume the dot
                            let field = self.parse_identifier()?;
                            expr = Expression::FieldAccess(FieldAccessExpression {
                                object: Box::new(expr),
                                field,
                            });
                        }
                        TokenKind::LBracket => {
                            self.next_token(); // consume the [
                            let index = self.parse_expression(Precedence::Lowest)?;
                            self.expect_and_consume(&TokenKind::RBracket)?;
                            expr = Expression::IndexAccess(IndexExpression {
                                array: Box::new(expr),
                                index: Box::new(index),
                            });
                        }
                        TokenKind::Question => {
                            self.next_token(); // consume the ?
                            expr = Expression::TryOperator(TryOperatorExpression {
                                expression: Box::new(expr),
                            });
                        }
                        _ => break,
                    }
                }

                Ok(expr)
            },
            TokenKind::Integer(val) => { self.next_token(); Ok(Expression::IntegerLiteral(*val)) },
            TokenKind::Float(val) => { self.next_token(); Ok(Expression::FloatLiteral(val.clone())) },
            TokenKind::String(val) => { self.next_token(); Ok(Expression::StringLiteral(val.clone())) },
            TokenKind::Bool(val) => { self.next_token(); Ok(Expression::BoolLiteral(*val)) },
            TokenKind::Minus | TokenKind::Bang => {
                // Parse prefix expressions: -x or !x
                let operator = token.clone();
                self.next_token();
                let right = self.parse_expression(Precedence::Product)?; // High precedence for prefix ops
                Ok(Expression::Prefix(PrefixExpression {
                    operator,
                    right: Box::new(right),
                }))
            },
            TokenKind::Ampersand => {
                // Parse borrow expression: &x or &mut x
                self.next_token();
                // Check for &mut x (mutable borrow)
                if self.consume_if_matches(&TokenKind::Mut) {
                    let expression = self.parse_expression(Precedence::Product)?;
                    Ok(Expression::MutableBorrow(MutableBorrowExpression {
                        expression: Box::new(expression),
                    }))
                } else {
                    // Otherwise it's &x (immutable borrow)
                    let expression = self.parse_expression(Precedence::Product)?;
                    Ok(Expression::Borrow(BorrowExpression {
                        expression: Box::new(expression),
                    }))
                }
            },
            TokenKind::Star => {
                // Parse dereference expression: *x
                self.next_token();
                let expression = self.parse_expression(Precedence::Product)?; // High precedence for prefix ops
                Ok(Expression::Dereference(DereferenceExpression {
                    expression: Box::new(expression),
                }))
            },
            TokenKind::LParen => self.parse_lambda_or_grouped(),
            TokenKind::LAngle => self.parse_jsx_element(),
            TokenKind::Pipe => self.parse_lambda_with_pipes(),
            TokenKind::LBracket => self.parse_array_literal(),
            TokenKind::Match => self.parse_match_expression(),
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

        // Try to determine if this is a lambda, tuple, or grouped expression
        if self.current_token().kind == TokenKind::RParen {
            // Empty parameter list for lambda: () =>
            self.expect_and_consume(&TokenKind::RParen)?;
            if self.consume_if_matches(&TokenKind::FatArrow) {
                let body = self.parse_expression(Precedence::Lowest)?;
                return Ok(Expression::Lambda(LambdaExpression {
                    parameters: vec![],
                    body: Box::new(body),
                    captures: vec![],  // Will be analyzed later
                }));
            }
            // Just empty parens, not lambda - error
            return Err(self.error("Unexpected empty parentheses"));
        }

        // Parse first expression
        let first_expr = self.parse_expression(Precedence::Lowest)?;

        // Check if this is a tuple (has comma after first element)
        if self.consume_if_matches(&TokenKind::Comma) {
            // This is a tuple: (expr, ...)
            let mut elements = vec![first_expr];

            // Parse remaining elements
            while self.current_token().kind != TokenKind::RParen {
                elements.push(self.parse_expression(Precedence::Lowest)?);
                if !self.consume_if_matches(&TokenKind::Comma) {
                    break;
                }
            }

            self.expect_and_consume(&TokenKind::RParen)?;
            return Ok(Expression::TupleLiteral(TupleLiteral { elements }));
        }

        // No comma, so it's either grouped expression or lambda
        self.expect_and_consume(&TokenKind::RParen)?;

        // Check if this is actually a lambda with single param: (x) => body
        if self.consume_if_matches(&TokenKind::FatArrow) {
            if let Expression::Identifier(param) = first_expr {
                let body = self.parse_expression(Precedence::Lowest)?;
                return Ok(Expression::Lambda(LambdaExpression {
                    parameters: vec![param],
                    body: Box::new(body),
                    captures: vec![],  // Will be analyzed later
                }));
            }
        }

        // Just a grouped expression
        Ok(first_expr)
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
            captures: vec![],  // Will be analyzed later
        }))
    }

    fn parse_array_literal(&mut self) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::LBracket)?;
        let mut elements = Vec::new();

        // Handle empty array []
        if self.current_token().kind == TokenKind::RBracket {
            self.expect_and_consume(&TokenKind::RBracket)?;
            return Ok(Expression::ArrayLiteral(ArrayLiteral { elements }));
        }

        // Parse comma-separated elements
        while self.current_token().kind != TokenKind::RBracket {
            elements.push(self.parse_expression(Precedence::Lowest)?);
            if !self.consume_if_matches(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_and_consume(&TokenKind::RBracket)?;
        Ok(Expression::ArrayLiteral(ArrayLiteral { elements }))
    }

    fn parse_struct_literal(&mut self, name: Identifier) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::LBrace)?;
        let mut fields = Vec::new();

        // Handle empty struct literal {}
        if self.current_token().kind == TokenKind::RBrace {
            self.expect_and_consume(&TokenKind::RBrace)?;
            return Ok(Expression::StructLiteral(StructLiteral { name, fields }));
        }

        // Parse comma-separated field: value pairs
        while self.current_token().kind != TokenKind::RBrace {
            let field_name = self.parse_identifier()?;
            self.expect_and_consume(&TokenKind::Colon)?;
            let field_value = self.parse_expression(Precedence::Lowest)?;
            fields.push((field_name, field_value));

            if !self.consume_if_matches(&TokenKind::Comma) {
                break;
            }
        }

        self.expect_and_consume(&TokenKind::RBrace)?;
        Ok(Expression::StructLiteral(StructLiteral { name, fields }))
    }

    fn parse_match_expression(&mut self) -> Result<Expression, CompileError> {
        self.expect_and_consume(&TokenKind::Match)?;

        // Parse the scrutinee (the value being matched)
        let scrutinee = Box::new(self.parse_expression(Precedence::Lowest)?);

        self.expect_and_consume(&TokenKind::LBrace)?;

        // Parse match arms
        let mut arms = Vec::new();
        while self.current_token().kind != TokenKind::RBrace {
            let pattern = self.parse_pattern()?;
            self.expect_and_consume(&TokenKind::FatArrow)?;
            let body = Box::new(self.parse_expression(Precedence::Lowest)?);

            arms.push(MatchArm { pattern, body });

            // Optionally consume comma between arms
            self.consume_if_matches(&TokenKind::Comma);
        }

        self.expect_and_consume(&TokenKind::RBrace)?;

        Ok(Expression::Match(MatchExpression { scrutinee, arms }))
    }

    fn parse_pattern(&mut self) -> Result<Pattern, CompileError> {
        let token = self.current_token().clone();

        match &token.kind {
            // Wildcard pattern: _
            TokenKind::Identifier if token.lexeme == "_" => {
                self.next_token();
                Ok(Pattern::Wildcard)
            }
            // Identifier (variable binding or enum variant)
            TokenKind::Identifier => {
                let first_ident = self.parse_identifier()?;

                // Check for :: (enum variant)
                if self.consume_if_matches(&TokenKind::DoubleColon) {
                    let variant_name = self.parse_identifier()?;

                    // Check for associated fields
                    let fields = if self.consume_if_matches(&TokenKind::LParen) {
                        let mut field_patterns = Vec::new();
                        while self.current_token().kind != TokenKind::RParen {
                            field_patterns.push(self.parse_pattern()?);
                            if !self.consume_if_matches(&TokenKind::Comma) { break; }
                        }
                        self.expect_and_consume(&TokenKind::RParen)?;
                        Some(field_patterns)
                    } else {
                        None
                    };

                    // Combine enum name and variant name
                    let full_name = Identifier {
                        value: format!("{}::{}", first_ident.value, variant_name.value)
                    };

                    Ok(Pattern::EnumVariant {
                        name: full_name,
                        fields,
                    })
                } else {
                    // Simple identifier binding
                    Ok(Pattern::Identifier(first_ident))
                }
            }
            // Literal patterns
            TokenKind::Integer(_) | TokenKind::Float(_) | TokenKind::String(_) |
            TokenKind::Bool(_) | TokenKind::True | TokenKind::False => {
                let literal_expr = self.parse_expression(Precedence::Lowest)?;
                Ok(Pattern::Literal(literal_expr))
            }
            _ => Err(self.error(&format!("Expected pattern, found {:?}", token.kind)))
        }
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