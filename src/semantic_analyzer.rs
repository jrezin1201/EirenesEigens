use crate::ast::*;
use crate::errors::CompileError;
use std::collections::HashMap;
use std::collections::HashSet;

/// Represents the fully resolved type of any expression in the compiler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedType {
    Integer,
    Float,
    String,
    Bool,
    Array(Box<ResolvedType>),
    Signal(Box<ResolvedType>), // Reactive signal wrapping a type
    Unit, // Represents "no value", like for a statement.
    Component, // For JSX components
    VNode, // For JSX elements (virtual DOM nodes)
    ComplexType,
    Unknown, // For types we haven't implemented yet
}

impl std::fmt::Display for ResolvedType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolvedType::Integer => write!(f, "i32"),
            ResolvedType::Float => write!(f, "f64"),
            ResolvedType::String => write!(f, "string"),
            ResolvedType::Bool => write!(f, "bool"),
            ResolvedType::Array(inner) => write!(f, "Array<{}>", inner),
            ResolvedType::Signal(inner) => write!(f, "Signal<{}>", inner),
            ResolvedType::Unit => write!(f, "()"),
            ResolvedType::Component => write!(f, "Component"),
            ResolvedType::VNode => write!(f, "VNode"),
            ResolvedType::ComplexType => write!(f, "complex"),
            ResolvedType::Unknown => write!(f, "unknown"),
        }
    }
}

impl ResolvedType {
    pub fn is_copy(&self) -> bool {
        matches!(self, ResolvedType::Integer | ResolvedType::Float | ResolvedType::Bool)
    }
}

/// A symbol table that manages scopes and declared variables.
struct SymbolTable {
    scopes: Vec<HashMap<String, ResolvedType>>,
}

impl SymbolTable {
    fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    fn define(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, ty);
    }

    fn lookup(&self, name: &str) -> Option<ResolvedType> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty.clone());
            }
        }
        None
    }
}

/// Traverses the AST to perform type checking and symbol resolution.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    in_component: bool,  // Track if we're inside a component
    reactive_variables: HashSet<String>,  // Track reactive variable names
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            in_component: false,
            reactive_variables: HashSet::new(),
        }
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<(), CompileError> {
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<ResolvedType, CompileError> {
        match stmt {
            Statement::Use(_) => Ok(ResolvedType::Unit),
            Statement::Let(let_stmt) => self.analyze_let_statement(let_stmt),
            Statement::Return(return_stmt) => self.analyze_return_statement(return_stmt),
            Statement::Expression(expr) => self.analyze_expression(expr),
            Statement::If(if_stmt) => self.analyze_if_statement(if_stmt),
            Statement::MacroInvocation(_) => Ok(ResolvedType::Unit),
            Statement::Function(_) => Ok(ResolvedType::Unit),
            Statement::Component(comp) => {
                // Mark that we're inside a component
                let was_in_component = self.in_component;
                self.in_component = true;

                // Analyze component body
                self.analyze_expression(&comp.body)?;

                self.in_component = was_in_component;
                Ok(ResolvedType::Component)
            }
            Statement::ExternBlock(_) => Ok(ResolvedType::Unit),
            Statement::Struct(_) => Ok(ResolvedType::Unit),
        }
    }

    fn analyze_if_statement(&mut self, stmt: &IfStatement) -> Result<ResolvedType, CompileError> {
        let cond_type = self.analyze_expression(&stmt.condition)?;
        if cond_type != ResolvedType::Bool && cond_type != ResolvedType::Integer {
            return Err(CompileError::Generic(format!(
                "If condition must be bool or integer, got '{}'",
                cond_type
            )));
        }

        // Analyze then branch
        for s in &stmt.then_branch.statements {
            self.analyze_statement(s)?;
        }

        // Analyze else branch if present
        if let Some(else_stmt) = &stmt.else_branch {
            self.analyze_statement(else_stmt)?;
        }

        Ok(ResolvedType::Unit)
    }

    fn analyze_let_statement(&mut self, stmt: &LetStatement) -> Result<ResolvedType, CompileError> {
        let mut value_type = self.analyze_expression(&stmt.value)?;

        // Auto-wrap in Signal<T> if inside a component and type is simple
        if self.in_component && self.should_be_reactive(&value_type) {
            value_type = ResolvedType::Signal(Box::new(value_type));
            self.reactive_variables.insert(stmt.name.value.clone());
            println!("[Reactive] Variable '{}' marked as reactive: {}", stmt.name.value, value_type);
        }

        self.symbols.define(stmt.name.value.clone(), value_type);
        Ok(ResolvedType::Unit)
    }

    fn should_be_reactive(&self, ty: &ResolvedType) -> bool {
        // Only wrap primitives in Signal<T> automatically
        matches!(ty,
            ResolvedType::Integer |
            ResolvedType::Float |
            ResolvedType::String |
            ResolvedType::Bool
        )
    }

    fn analyze_return_statement(&mut self, stmt: &ReturnStatement) -> Result<ResolvedType, CompileError> {
        self.analyze_expression(&stmt.value)?;
        Ok(ResolvedType::Unit)
    }

    fn analyze_expression(&mut self, expr: &Expression) -> Result<ResolvedType, CompileError> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(ResolvedType::Integer),
            Expression::FloatLiteral(_) => Ok(ResolvedType::Float),
            Expression::StringLiteral(_) => Ok(ResolvedType::String),
            Expression::BoolLiteral(_) => Ok(ResolvedType::Bool),
            Expression::Identifier(ident) => {
                if let Some(ty) = self.symbols.lookup(&ident.value) {
                    Ok(ty)
                } else {
                    Ok(ResolvedType::ComplexType)
                }
            }
            Expression::Infix(infix_expr) => self.analyze_infix_expression(infix_expr),
            Expression::JsxElement(_) => Ok(ResolvedType::VNode),
            Expression::FunctionCall(_) => Ok(ResolvedType::Unknown),
            Expression::Lambda(_) => Ok(ResolvedType::Unknown),
        }
    }

    fn analyze_infix_expression(&mut self, expr: &InfixExpression) -> Result<ResolvedType, CompileError> {
        let left_type = self.analyze_expression(&expr.left)?;
        let right_type = self.analyze_expression(&expr.right)?;

        if left_type == ResolvedType::Integer && right_type == ResolvedType::Integer {
            Ok(ResolvedType::Integer)
        } else {
            Err(CompileError::Generic(format!(
                "Cannot apply operator '{}' to types '{}' and '{}'",
                expr.operator.lexeme, left_type, right_type
            )))
        }
    }
}