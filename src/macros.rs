// src/macros.rs
use crate::token::{Token, TokenKind};

// This function now parses props from the component signature.
pub fn expand_component_macro(input: &[Token]) -> Result<Vec<Token>, String> {
    // A simple parser to extract name and props.
    // A real macro system would have more robust tools for this.
    let component_name = &input.get(0).ok_or("Expected component name")?.lexeme;
    let props_struct_name = format!("{}Props", component_name);
    
    // Find the tokens inside the `(...)` for props.
    let lparen = input.iter().position(|t| t.kind == TokenKind::LParen).unwrap_or(0);
    let rparen = input.iter().position(|t| t.kind == TokenKind::RParen).unwrap_or(lparen);
    let prop_tokens = &input[lparen + 1..rparen];

    let mut output_tokens = Vec::new();

    // 1. Generate: struct MyComponentProps { ... fields ... }
    output_tokens.push(Token::new(TokenKind::Struct, "struct".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Identifier, props_struct_name.clone(), 0, 0));
    output_tokens.push(Token::new(TokenKind::LBrace, "{".to_string(), 0, 0));
    // Add the parsed props as fields in the struct.
    output_tokens.extend_from_slice(prop_tokens);
    output_tokens.push(Token::new(TokenKind::RBrace, "}".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Semicolon, ";".to_string(), 0, 0));

    // 2. Generate: fn MyComponent(props: MyComponentProps) -> VNode { ... body ... }
    output_tokens.push(Token::new(TokenKind::Fn, "fn".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Identifier, component_name.clone(), 0, 0));
    output_tokens.push(Token::new(TokenKind::LParen, "(".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Identifier, "props".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Colon, ":".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Identifier, props_struct_name, 0, 0));
    output_tokens.push(Token::new(TokenKind::RParen, ")".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Arrow, "->".to_string(), 0, 0));
    output_tokens.push(Token::new(TokenKind::Identifier, "VNode".to_string(), 0, 0));

    // 3. Append the original function body.
    let body_start = input.iter().position(|t| t.kind == TokenKind::LBrace).unwrap_or(0);
    output_tokens.extend_from_slice(&input[body_start..]);

    Ok(output_tokens)
}