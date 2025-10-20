use crate::token::Token;

#[derive(Debug, Clone)]
pub struct Program {
    pub statements: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Use(UseStatement),
    Let(LetStatement),
    Assignment(AssignmentStatement),
    Return(ReturnStatement),
    Expression(Expression),
    If(IfStatement),
    While(WhileStatement),
    For(ForStatement),
    ForIn(ForInStatement),
    MacroInvocation(MacroInvocation),
    Struct(StructDefinition),
    Enum(EnumDefinition),
    Function(FunctionDefinition),
    Component(ComponentDefinition),
    ExternBlock(ExternBlock),
    ImplBlock(ImplBlock),
    Trait(TraitDefinition),
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
pub struct AssignmentStatement {
    pub target: Identifier,
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
pub struct WhileStatement {
    pub condition: Expression,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct ForStatement {
    pub init: Option<Box<Statement>>,  // Optional initialization (let i = 0)
    pub condition: Expression,          // Loop condition
    pub update: Option<Box<Statement>>, // Optional update (i = i + 1)
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct ForInStatement {
    pub variable: Identifier,          // Loop variable (e.g., "item" in "for item in collection")
    pub iterator: Expression,          // The expression to iterate over
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct MacroInvocation {
    pub name: Identifier,
    pub input_tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct StructDefinition {
    pub name: Identifier,
    pub lifetime_params: Vec<Lifetime>,  // Lifetime parameters like <'a, 'b>
    pub type_params: Vec<Identifier>,  // Generic type parameters like <T, U>
    pub fields: Vec<(Identifier, TypeExpression)>,
}

#[derive(Debug, Clone)]
pub struct EnumDefinition {
    pub name: Identifier,
    pub lifetime_params: Vec<Lifetime>,  // Lifetime parameters like <'a, 'b>
    pub type_params: Vec<Identifier>,  // Generic type parameters like <T, U>
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: Identifier,
    pub fields: Option<Vec<(Identifier, TypeExpression)>>,  // For tuple/struct variants
}

#[derive(Debug, Clone)]
pub struct FunctionDefinition {
    pub name: Identifier,
    pub lifetime_params: Vec<Lifetime>,  // Lifetime parameters like <'a, 'b>
    pub type_params: Vec<Identifier>,  // Generic type parameters like <T, U>
    pub parameters: Vec<FunctionParameter>,
    pub is_server: bool,
    pub is_client: bool,
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
    ArrayLiteral(ArrayLiteral),
    TupleLiteral(TupleLiteral),
    StructLiteral(StructLiteral),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    FieldAccess(FieldAccessExpression),
    IndexAccess(IndexExpression),
    Match(MatchExpression),
    JsxElement(JsxElement),
    FunctionCall(FunctionCall),
    Lambda(LambdaExpression),
    Borrow(BorrowExpression),      // &x (create reference)
    MutableBorrow(MutableBorrowExpression),  // &mut x (create mutable reference)
    Dereference(DereferenceExpression),  // *x (dereference)
    Range(RangeExpression),  // start..end or start..=end
    TryOperator(TryOperatorExpression),  // expr? (error propagation)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Identifier {
    pub value: String,
}

// Lifetime annotation like 'a, 'b, 'static
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Lifetime {
    pub name: String,  // e.g., "a" for 'a, "static" for 'static
}

#[derive(Debug, Clone)]
pub struct PrefixExpression {
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct InfixExpression {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct BorrowExpression {
    pub expression: Box<Expression>,  // The expression being borrowed
}

#[derive(Debug, Clone)]
pub struct MutableBorrowExpression {
    pub expression: Box<Expression>,  // The expression being mutably borrowed
}

#[derive(Debug, Clone)]
pub struct DereferenceExpression {
    pub expression: Box<Expression>,  // The expression being dereferenced
}

#[derive(Debug, Clone)]
pub struct RangeExpression {
    pub start: Option<Box<Expression>>,  // Start of range (None for ..end)
    pub end: Option<Box<Expression>>,    // End of range (None for start..)
    pub inclusive: bool,                  // true for ..=, false for ..
}

#[derive(Debug, Clone)]
pub struct TryOperatorExpression {
    pub expression: Box<Expression>,  // The expression being tried (must return Result<T, E>)
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct TupleLiteral {
    pub elements: Vec<Expression>,
}

#[derive(Debug, Clone)]
pub struct StructLiteral {
    pub name: Identifier,
    pub fields: Vec<(Identifier, Expression)>,
}

#[derive(Debug, Clone)]
pub struct FieldAccessExpression {
    pub object: Box<Expression>,
    pub field: Identifier,
}

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct MatchExpression {
    pub scrutinee: Box<Expression>,  // The value being matched
    pub arms: Vec<MatchArm>,
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Box<Expression>,
}

#[derive(Debug, Clone)]
pub enum Pattern {
    Identifier(Identifier),           // x (binds to variable)
    Literal(Expression),              // 42, "hello", true
    Wildcard,                         // _ (matches anything)
    EnumVariant {
        name: Identifier,             // Color::Red or Option::Some
        fields: Option<Vec<Pattern>>, // For destructuring fields
    },
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
    Tuple(Vec<TypeExpression>),  // (i32, String, bool)
    Reference(Box<TypeExpression>),  // &T (immutable reference)
    MutableReference(Box<TypeExpression>),  // &mut T (mutable reference)
    Slice(Box<TypeExpression>),  // [T] (slice type)
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
pub enum CaptureMode {
    ByReference,       // Capture by immutable reference (&)
    ByMutableReference, // Capture by mutable reference (&mut)
    ByValue,           // Capture by value (move)
}

#[derive(Debug, Clone)]
pub struct CapturedVariable {
    pub name: Identifier,
    pub mode: CaptureMode,
}

#[derive(Debug, Clone)]
pub struct LambdaExpression {
    pub parameters: Vec<Identifier>,
    pub body: Box<Expression>,
    pub captures: Vec<CapturedVariable>,  // Variables captured from environment
}

#[derive(Debug, Clone)]
pub struct ComponentDefinition {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub is_client: bool,  // Components are client-side by default
    pub body: Box<Expression>,
}

#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub trait_name: Option<Identifier>,  // None for inherent impl, Some for trait impl
    pub lifetime_params: Vec<Lifetime>,  // Lifetime parameters like <'a, 'b>
    pub type_params: Vec<Identifier>,  // Generic type parameters like <T, U>
    pub type_name: Identifier,  // The type being implemented (e.g., "Point")
    pub methods: Vec<ImplMethod>,
}

#[derive(Debug, Clone)]
pub struct ImplMethod {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,  // First parameter is usually &self or self
    pub return_type: Option<TypeExpression>,
    pub body: BlockStatement,
}

#[derive(Debug, Clone)]
pub struct TraitDefinition {
    pub name: Identifier,
    pub lifetime_params: Vec<Lifetime>,  // Lifetime parameters like <'a, 'b>
    pub type_params: Vec<Identifier>,  // Generic type parameters like <T, U>
    pub methods: Vec<TraitMethod>,
}

#[derive(Debug, Clone)]
pub struct TraitMethod {
    pub name: Identifier,
    pub parameters: Vec<FunctionParameter>,
    pub return_type: Option<TypeExpression>,
}