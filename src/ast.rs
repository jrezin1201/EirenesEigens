use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Use(UseStatement),
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(Expression),
    If(IfStatement),
    MacroInvocation(MacroInvocation),
    Struct(StructDefinition),
    Function(FunctionDefinition),
    Component(ComponentDefinition),
    ExternBlock(ExternBlock),
}

#[derive(Debug, Clone)]
pub struct UseStatement {
    pub path: Vec<Identifier>,
    pub imports: Vec<Identifier>,
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: BlockStatement,
    pub else_branch: Option<Box<Statement>>,  // Can be another if or block
}

#[derive(Debug, Clone)]
pub struct MacroInvocation {
    pub name: Identifier,
    pub input_tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: Identifier,
    pub fields: Vec<(Identifier, TypeExpression)>,
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub is_server: bool,
    pub is_async: bool,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub struct ExternBlock {
    pub abi: String,
    pub functions: Vec<FunctionDeclaration>,
}

#[derive(Debug, Clone)]
pub struct FunctionDeclaration {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeExpression>,
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(i64),
    FloatLiteral(String),
    StringLiteral(String),
    BoolLiteral(bool),
    Infix(InfixExpression),
    JsxElement(JsxElement),
    FunctionCall(FunctionCall),
    Lambda(LambdaExpression),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub value: String,
}

#[derive(Debug, Clone)]
pub struct InfixExpression {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct FunctionParameter {
    pub name: Identifier,
    pub type_annotation: TypeExpression,
}

// This is the single, correct definition for TypeExpression
#[derive(Debug, Clone)]
pub enum TypeExpression {
    Named(Identifier),
    Generic(Identifier, Vec<TypeExpression>),
}

// --- JSX AST Nodes ---
#[derive(Debug, Clone)]
pub struct JsxElement {
    pub opening_tag: JsxOpeningTag,
    pub children: Vec<JsxChild>,
    pub closing_tag: Option<Identifier>,
}

#[derive(Debug, Clone)]
pub struct JsxOpeningTag {
    pub name: Identifier,
    pub attributes: Vec<JsxAttribute>,
    pub self_closing: bool,
}

#[derive(Debug, Clone)]
pub enum JsxChild {
    Element(Box<JsxElement>),
    Text(String),
    Expression(Box<Expression>), // For {expr} interpolation
}

#[derive(Debug, Clone)]
pub struct JsxAttribute {
    pub name: Identifier,
    pub value: Expression,
}

#[derive(Debug, Clone)]
pub struct FunctionCall {
    pub function: Box<Expression>,
    pub arguments: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct LambdaExpression {
    pub parameters: Vec<Identifier>,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub body: Box<Expression>,
}