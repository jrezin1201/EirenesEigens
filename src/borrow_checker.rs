use crate::ast::*;
use crate::errors::CompileError;
use crate::semantic_analyzer::ResolvedType;
use std::collections::HashMap;

/// Represents the current state of a variable's ownership.
#[derive(Debug, Clone)]
enum OwnershipState {
    Owned, // The variable is valid and owns its data.
    Moved, // The variable's data has been moved to another owner.
}

/// A symbol table that tracks ownership state in addition to types.
struct BorrowSymbolTable {
    scopes: Vec<HashMap<String, (OwnershipState, ResolvedType)>>,
}

impl BorrowSymbolTable {
    fn new() -> Self { Self { scopes: vec![HashMap::new()] } }

    fn define(&mut self, name: String, ty: ResolvedType) {
        self.scopes.last_mut().unwrap().insert(name, (OwnershipState::Owned, ty));
    }

    fn update_state(&mut self, name: &str, new_state: OwnershipState) {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                symbol.0 = new_state;
                return;
            }
        }
    }

    fn lookup(&self, name: &str) -> Option<(OwnershipState, ResolvedType)> {
        for scope in self.scopes.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol.clone());
            }
        }
        None
    }
}

/// Traverses a type-checked AST to enforce ownership rules.
pub struct BorrowChecker {
    symbols: BorrowSymbolTable,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self { symbols: BorrowSymbolTable::new() }
    }

    pub fn check_program(&mut self, program: &Program) -> Result<(), CompileError> {
        for stmt in &program.statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    fn check_statement(&mut self, stmt: &Statement) -> Result<(), CompileError> {
        match stmt {
            Statement::Use(_) => Ok(()),
            Statement::Let(let_stmt) => self.check_let_statement(let_stmt),
            Statement::Return(return_stmt) => self.check_expression(&return_stmt.value).map(|_| ()),
            Statement::Expression(expr) => self.check_expression(expr).map(|_| ()),
            Statement::If(if_stmt) => self.check_if_statement(if_stmt),
            Statement::MacroInvocation(_) => Ok(()),
            Statement::Function(_) => Ok(()),
            Statement::Component(_) => Ok(()),
            Statement::ExternBlock(_) => Ok(()),
            Statement::Struct(_) => Ok(()),
        }
    }

    fn check_if_statement(&mut self, stmt: &IfStatement) -> Result<(), CompileError> {
        self.check_expression(&stmt.condition)?;
        for s in &stmt.then_branch.statements {
            self.check_statement(s)?;
        }
        if let Some(else_stmt) = &stmt.else_branch {
            self.check_statement(else_stmt)?;
        }
        Ok(())
    }
    
    fn check_let_statement(&mut self, stmt: &LetStatement) -> Result<(), CompileError> {
        let value_type = self.check_expression(&stmt.value)?;
        if let Expression::Identifier(ident) = &stmt.value {
            if !value_type.is_copy() {
                self.symbols.update_state(&ident.value, OwnershipState::Moved);
            }
        }
        self.symbols.define(stmt.name.value.clone(), value_type);
        Ok(())
    }

    fn check_expression(&mut self, expr: &Expression) -> Result<ResolvedType, CompileError> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(ResolvedType::Integer),
            Expression::FloatLiteral(_) => Ok(ResolvedType::Float),
            Expression::BoolLiteral(_) => Ok(ResolvedType::Bool),
            Expression::StringLiteral(_) => Ok(ResolvedType::String),
            Expression::Identifier(ident) => {
                let (state, ty) = self.symbols.lookup(&ident.value)
                    .ok_or_else(|| CompileError::Generic(format!("Borrow checker: undefined variable '{}'", ident.value)))?;

                if let OwnershipState::Moved = state {
                    return Err(CompileError::BorrowError(format!("Use of moved value: '{}'", ident.value)));
                }
                Ok(ty)
            }
            Expression::Infix(infix_expr) => {
                self.check_expression(&infix_expr.left)?;
                self.check_expression(&infix_expr.right)?;
                Ok(ResolvedType::Integer)
            }
            Expression::JsxElement(_) => Ok(ResolvedType::VNode),
            Expression::FunctionCall(_) => Ok(ResolvedType::Unknown),
            Expression::Lambda(_) => Ok(ResolvedType::Unknown),
        }
    }
}