// Type Checker with Hindley-Milner Type Inference

use crate::ast::{Expression, Statement, InfixExpression};
use crate::errors::CompileError;
use crate::types::{Substitution, Type, TypeEnv};
use std::collections::HashSet;

pub struct TypeChecker {
    env: TypeEnv,
    constraints: Vec<(Type, Type)>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut env = TypeEnv::new();

        // Add built-in types and functions
        env.bind("console".to_string(), Type::Any);
        env.bind("Math".to_string(), Type::Any);

        TypeChecker {
            env,
            constraints: Vec::new(),
        }
    }

    /// Type check a program (list of statements)
    pub fn check_program(&mut self, statements: &[Statement]) -> Result<(), CompileError> {
        for stmt in statements {
            self.check_statement(stmt)?;
        }
        Ok(())
    }

    /// Infer the type of a statement
    pub fn check_statement(&mut self, stmt: &Statement) -> Result<Type, CompileError> {
        match stmt {
            Statement::Let(let_stmt) => {
                let value_type = self.infer_expression(&let_stmt.value)?;
                self.env.bind(let_stmt.name.value.clone(), value_type.clone());
                Ok(value_type)
            }

            Statement::Function(func_def) => {
                self.env.push_scope();

                // Check body
                let mut body_type = Type::Void;
                for stmt in &func_def.body.statements {
                    body_type = self.check_statement(stmt)?;
                }

                self.env.pop_scope();

                let func_type = Type::function(vec![], body_type);
                self.env.bind(func_def.name.value.clone(), func_type.clone());
                Ok(func_type)
            }

            Statement::Component(comp_def) => {
                self.env.push_scope();

                // Bind parameters
                for param in &comp_def.parameters {
                    self.env.bind(param.name.value.clone(), Type::Any);
                }

                // Check body
                let _body_type = self.infer_expression(&comp_def.body)?;

                self.env.pop_scope();

                let component_type = Type::Component(vec![]);
                self.env.bind(comp_def.name.value.clone(), component_type.clone());
                Ok(component_type)
            }

            Statement::Return(ret_stmt) => {
                self.infer_expression(&ret_stmt.value)
            }

            Statement::Expression(expr) => self.infer_expression(expr),

            Statement::If(if_stmt) => {
                let cond_type = self.infer_expression(&if_stmt.condition)?;
                if cond_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "If condition must be bool, got {}",
                        cond_type
                    )));
                }

                for stmt in &if_stmt.then_branch.statements {
                    self.check_statement(stmt)?;
                }

                if let Some(else_branch) = &if_stmt.else_branch {
                    self.check_statement(else_branch)?;
                }

                Ok(Type::Void)
            }

            _ => Ok(Type::Void),
        }
    }

    /// Infer the type of an expression using Hindley-Milner algorithm
    pub fn infer_expression(&mut self, expr: &Expression) -> Result<Type, CompileError> {
        match expr {
            Expression::IntegerLiteral(_) => Ok(Type::Int),
            Expression::FloatLiteral(_) => Ok(Type::Float),
            Expression::StringLiteral(_) => Ok(Type::String),
            Expression::BoolLiteral(_) => Ok(Type::Bool),

            Expression::Identifier(ident) => {
                if let Some(ty) = self.env.lookup(&ident.value) {
                    Ok(ty.clone())
                } else {
                    Err(CompileError::Generic(format!(
                        "Undefined variable: {}",
                        ident.value
                    )))
                }
            }

            Expression::Infix(infix) => {
                self.check_infix_expression(infix)
            }

            Expression::FunctionCall(call) => {
                // Infer function type
                let _func_type = self.infer_expression(&call.function)?;

                // For now, return Any for function calls
                // TODO: Implement proper function type checking
                Ok(Type::Any)
            }

            Expression::JsxElement(_) => {
                // JSX elements return component instances
                Ok(Type::Named("ReactElement".to_string()))
            }

            Expression::Lambda(lambda) => {
                self.env.push_scope();

                // Bind parameters
                for param in &lambda.parameters {
                    self.env.bind(param.value.clone(), Type::Any);
                }

                let body_type = self.infer_expression(&lambda.body)?;

                self.env.pop_scope();

                Ok(Type::function(vec![Type::Any; lambda.parameters.len()], body_type))
            }

            _ => Ok(Type::Any),
        }
    }

    fn check_infix_expression(&mut self, infix: &InfixExpression) -> Result<Type, CompileError> {
        let left_type = self.infer_expression(&infix.left)?;
        let right_type = self.infer_expression(&infix.right)?;

        let op = &infix.operator.lexeme;

        match op.as_str() {
            "+" | "-" | "*" | "/" | "%" => {
                // Arithmetic operations
                if !left_type.is_numeric() {
                    return Err(CompileError::Generic(format!(
                        "Expected numeric type, got {}",
                        left_type
                    )));
                }
                if !right_type.is_numeric() {
                    return Err(CompileError::Generic(format!(
                        "Expected numeric type, got {}",
                        right_type
                    )));
                }

                // Result is Float if either operand is Float
                if left_type == Type::Float || right_type == Type::Float {
                    Ok(Type::Float)
                } else {
                    Ok(Type::Int)
                }
            }

            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                // Comparison operations
                if !left_type.is_compatible_with(&right_type) {
                    return Err(CompileError::Generic(format!(
                        "Cannot compare {} with {}",
                        left_type, right_type
                    )));
                }
                Ok(Type::Bool)
            }

            "&&" | "||" => {
                // Logical operations
                if left_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "Expected bool, got {}",
                        left_type
                    )));
                }
                if right_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "Expected bool, got {}",
                        right_type
                    )));
                }
                Ok(Type::Bool)
            }

            _ => Ok(Type::Any),
        }
    }

    /// Unify two types and generate substitutions
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<Substitution, CompileError> {
        match (t1, t2) {
            // Same types unify trivially
            (a, b) if a == b => Ok(Substitution::new()),

            // Any unifies with everything
            (Type::Any, _) | (_, Type::Any) => Ok(Substitution::new()),

            // Type variables
            (Type::Var(id), ty) | (ty, Type::Var(id)) => {
                if self.occurs_check(*id, ty) {
                    Err(CompileError::Generic("Infinite type detected".to_string()))
                } else {
                    let mut subst = Substitution::new();
                    subst.insert(*id, ty.clone());
                    Ok(subst)
                }
            }

            // Functions
            (
                Type::Function {
                    params: p1,
                    return_type: r1,
                },
                Type::Function {
                    params: p2,
                    return_type: r2,
                },
            ) => {
                if p1.len() != p2.len() {
                    return Err(CompileError::Generic("Function arity mismatch".to_string()));
                }

                let mut subst = Substitution::new();

                // Unify parameters
                for (param1, param2) in p1.iter().zip(p2.iter()) {
                    let s = self.unify(&subst.apply(param1), &subst.apply(param2))?;
                    subst = subst.compose(&s);
                }

                // Unify return types
                let s = self.unify(&subst.apply(r1), &subst.apply(r2))?;
                Ok(subst.compose(&s))
            }

            // Arrays
            (Type::Array(t1), Type::Array(t2)) => self.unify(t1, t2),

            // Options
            (Type::Option(t1), Type::Option(t2)) => self.unify(t1, t2),

            // Tuples
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                if t1.len() != t2.len() {
                    return Err(CompileError::Generic("Tuple length mismatch".to_string()));
                }

                let mut subst = Substitution::new();
                for (ty1, ty2) in t1.iter().zip(t2.iter()) {
                    let s = self.unify(&subst.apply(ty1), &subst.apply(ty2))?;
                    subst = subst.compose(&s);
                }
                Ok(subst)
            }

            // Numeric compatibility
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => Ok(Substitution::new()),

            _ => Err(CompileError::Generic(format!(
                "Cannot unify types {} and {}",
                t1, t2
            ))),
        }
    }

    /// Occurs check - prevent infinite types
    fn occurs_check(&self, var: usize, ty: &Type) -> bool {
        match ty {
            Type::Var(id) => *id == var,
            Type::Array(inner) => self.occurs_check(var, inner),
            Type::Option(inner) => self.occurs_check(var, inner),
            Type::Function { params, return_type } => {
                params.iter().any(|p| self.occurs_check(var, p))
                    || self.occurs_check(var, return_type)
            }
            Type::Tuple(types) => types.iter().any(|t| self.occurs_check(var, t)),
            Type::Union(types) => types.iter().any(|t| self.occurs_check(var, t)),
            _ => false,
        }
    }

    /// Solve all constraints and return final substitution
    pub fn solve_constraints(&mut self) -> Result<Substitution, CompileError> {
        let mut subst = Substitution::new();

        for (t1, t2) in &self.constraints.clone() {
            let s = self.unify(&subst.apply(t1), &subst.apply(t2))?;
            subst = subst.compose(&s);
        }

        Ok(subst)
    }

    /// Get free type variables in a type
    #[allow(dead_code)]
    fn free_vars(&self, ty: &Type) -> HashSet<usize> {
        match ty {
            Type::Var(id) => {
                let mut set = HashSet::new();
                set.insert(*id);
                set
            }
            Type::Array(inner) => self.free_vars(inner),
            Type::Option(inner) => self.free_vars(inner),
            Type::Function { params, return_type } => {
                let mut set = HashSet::new();
                for param in params {
                    set.extend(self.free_vars(param));
                }
                set.extend(self.free_vars(return_type));
                set
            }
            Type::Tuple(types) => {
                let mut set = HashSet::new();
                for ty in types {
                    set.extend(self.free_vars(ty));
                }
                set
            }
            Type::Union(types) => {
                let mut set = HashSet::new();
                for ty in types {
                    set.extend(self.free_vars(ty));
                }
                set
            }
            _ => HashSet::new(),
        }
    }
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unify_primitives() {
        let mut checker = TypeChecker::new();
        assert!(checker.unify(&Type::Int, &Type::Int).is_ok());
        assert!(checker.unify(&Type::Int, &Type::Float).is_ok());
        assert!(checker.unify(&Type::String, &Type::Bool).is_err());
    }

    #[test]
    fn test_unify_type_variables() {
        let mut checker = TypeChecker::new();
        let result = checker.unify(&Type::Var(0), &Type::Int);
        assert!(result.is_ok());

        let subst = result.unwrap();
        assert_eq!(subst.apply(&Type::Var(0)), Type::Int);
    }

    #[test]
    fn test_occurs_check() {
        let checker = TypeChecker::new();
        let recursive_type = Type::Array(Box::new(Type::Var(0)));
        assert!(checker.occurs_check(0, &recursive_type));
    }
}
