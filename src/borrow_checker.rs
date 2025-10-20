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
            Statement::Assignment(assign_stmt) => {
                // Check that the target variable exists
                if self.symbols.lookup(&assign_stmt.target.value).is_none() {
                    return Err(CompileError::Generic(format!(
                        "Cannot assign to undefined variable '{}'",
                        assign_stmt.target.value
                    )));
                }

                // Check the value expression
                self.check_expression(&assign_stmt.value)?;

                // If the value is moved from a variable, update its state
                if let Expression::Identifier(ident) = &assign_stmt.value {
                    if let Some((_, ty)) = self.symbols.lookup(&ident.value) {
                        if !ty.is_copy() {
                            self.symbols.update_state(&ident.value, OwnershipState::Moved);
                        }
                    }
                }

                Ok(())
            }
            Statement::Return(return_stmt) => self.check_expression(&return_stmt.value).map(|_| ()),
            Statement::Expression(expr) => self.check_expression(expr).map(|_| ()),
            Statement::If(if_stmt) => self.check_if_statement(if_stmt),
            Statement::While(while_stmt) => self.check_while_statement(while_stmt),
            Statement::For(for_stmt) => self.check_for_statement(for_stmt),
            Statement::ForIn(for_in_stmt) => self.check_for_in_statement(for_in_stmt),
            Statement::MacroInvocation(_) => Ok(()),
            Statement::Function(_) => Ok(()),
            Statement::Component(_) => Ok(()),
            Statement::ExternBlock(_) => Ok(()),
            Statement::Struct(_) => Ok(()),
            Statement::Enum(_) => Ok(()),
            Statement::ImplBlock(_) => Ok(()),
            Statement::Trait(_) => Ok(()),
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

    fn check_while_statement(&mut self, stmt: &WhileStatement) -> Result<(), CompileError> {
        self.check_expression(&stmt.condition)?;
        for s in &stmt.body.statements {
            self.check_statement(s)?;
        }
        Ok(())
    }

    fn check_for_statement(&mut self, stmt: &ForStatement) -> Result<(), CompileError> {
        // Check init statement if present
        if let Some(init) = &stmt.init {
            self.check_statement(init)?;
        }

        // Check condition
        self.check_expression(&stmt.condition)?;

        // Check update statement if present
        if let Some(update) = &stmt.update {
            self.check_statement(update)?;
        }

        // Check body
        for s in &stmt.body.statements {
            self.check_statement(s)?;
        }

        Ok(())
    }

    fn check_for_in_statement(&mut self, stmt: &ForInStatement) -> Result<(), CompileError> {
        // Check the iterator expression
        self.check_expression(&stmt.iterator)?;

        // Check body statements
        for s in &stmt.body.statements {
            self.check_statement(s)?;
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
            Expression::Prefix(prefix_expr) => {
                self.check_expression(&prefix_expr.right)?;
                Ok(ResolvedType::Integer)
            }
            Expression::Infix(infix_expr) => {
                self.check_expression(&infix_expr.left)?;
                self.check_expression(&infix_expr.right)?;
                Ok(ResolvedType::Integer)
            }
            Expression::ArrayLiteral(array_lit) => {
                // Check all elements
                if array_lit.elements.is_empty() {
                    Ok(ResolvedType::Array(Box::new(ResolvedType::Unknown)))
                } else {
                    let first_type = self.check_expression(&array_lit.elements[0])?;
                    for elem in &array_lit.elements[1..] {
                        self.check_expression(elem)?;
                    }
                    Ok(ResolvedType::Array(Box::new(first_type)))
                }
            }
            Expression::TupleLiteral(tuple_lit) => {
                // Check all tuple elements
                let mut element_types = Vec::new();
                for elem in &tuple_lit.elements {
                    let elem_type = self.check_expression(elem)?;
                    element_types.push(elem_type);
                }
                Ok(ResolvedType::Tuple(element_types))
            }
            Expression::StructLiteral(struct_lit) => {
                // Check all field values
                for (_field_name, field_value) in &struct_lit.fields {
                    self.check_expression(field_value)?;
                }
                // For now, return Unknown type
                Ok(ResolvedType::Unknown)
            }
            Expression::FieldAccess(field_access) => {
                // Check the object expression
                self.check_expression(&field_access.object)?;
                // For now, return Unknown type
                Ok(ResolvedType::Unknown)
            }
            Expression::Match(match_expr) => {
                // Check the scrutinee
                self.check_expression(&match_expr.scrutinee)?;

                // Check all match arms
                for arm in &match_expr.arms {
                    self.check_expression(&arm.body)?;
                }

                // For now, return Unknown type
                Ok(ResolvedType::Unknown)
            }
            Expression::IndexAccess(index_expr) => {
                // Check the array expression
                let array_type = self.check_expression(&index_expr.array)?;

                // Check the index expression
                self.check_expression(&index_expr.index)?;

                // Return the element type if it's an array
                match array_type {
                    ResolvedType::Array(element_type) => Ok(*element_type),
                    _ => Ok(ResolvedType::Unknown),
                }
            }
            Expression::JsxElement(_) => Ok(ResolvedType::VNode),
            Expression::FunctionCall(_) => Ok(ResolvedType::Unknown),
            Expression::Lambda(_) => Ok(ResolvedType::Unknown),
            Expression::Borrow(borrow_expr) => {
                self.check_expression(&borrow_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::MutableBorrow(borrow_expr) => {
                self.check_expression(&borrow_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::Dereference(deref_expr) => {
                self.check_expression(&deref_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::Range(_range_expr) => {
                // Range expressions are placeholders for now
                // In a full implementation, we'd check the start and end expressions
                Ok(ResolvedType::Unknown)
            }
            Expression::TryOperator(try_expr) => {
                self.check_expression(&try_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
        }
    }
}