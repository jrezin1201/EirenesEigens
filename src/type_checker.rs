// Type Checker with Hindley-Milner Type Inference

use crate::ast::{Expression, Statement, InfixExpression, PrefixExpression, TypeExpression};
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

    /// Convert TypeExpression from AST to Type
    fn type_expr_to_type(&self, type_expr: &TypeExpression) -> Type {
        match type_expr {
            TypeExpression::Named(ident) => {
                match ident.value.as_str() {
                    "i32" | "i64" | "i8" | "i16" | "isize" => Type::Int,
                    "f32" | "f64" => Type::Float,
                    "bool" => Type::Bool,
                    "str" | "String" => Type::String,
                    _ => Type::Named(ident.value.clone()),
                }
            }
            TypeExpression::Generic(ident, args) => {
                match ident.value.as_str() {
                    "Array" | "Vec" if args.len() == 1 => {
                        Type::Array(Box::new(self.type_expr_to_type(&args[0])))
                    }
                    "Option" if args.len() == 1 => {
                        Type::Option(Box::new(self.type_expr_to_type(&args[0])))
                    }
                    _ => Type::Named(ident.value.clone()),
                }
            }
            TypeExpression::Tuple(types) => {
                // Convert tuple type expression to Type::Tuple
                let converted_types: Vec<Type> = types.iter()
                    .map(|t| self.type_expr_to_type(t))
                    .collect();
                Type::Tuple(converted_types)
            }
            TypeExpression::Reference(_inner) => {
                // For now, return Any for reference types
                Type::Any
            }
            TypeExpression::MutableReference(_inner) => {
                // For now, return Any for mutable reference types
                Type::Any
            }
            TypeExpression::Slice(inner) => {
                // Slices are array views - recursively process the inner type
                // Return Type::Array with the inner type
                let inner_type = self.type_expr_to_type(inner);
                Type::Array(Box::new(inner_type))
            }
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

            Statement::Assignment(assign_stmt) => {
                // Check that variable exists and clone the type
                let var_type = self.env.lookup(&assign_stmt.target.value)
                    .ok_or_else(|| CompileError::Generic(format!(
                        "Cannot assign to undefined variable '{}'",
                        assign_stmt.target.value
                    )))?.clone();

                // Check that value type matches variable type
                let value_type = self.infer_expression(&assign_stmt.value)?;
                if let Err(e) = self.unify(&value_type, &var_type) {
                    return Err(CompileError::Generic(format!(
                        "Type mismatch in assignment to '{}': expected {}, got {}. {}",
                        assign_stmt.target.value, var_type, value_type, e
                    )));
                }

                Ok(Type::Void)
            }

            Statement::Function(func_def) => {
                self.env.push_scope();

                // Bind parameters to scope with their actual types
                let mut param_types = Vec::new();
                for param in &func_def.parameters {
                    let param_type = self.type_expr_to_type(&param.type_annotation);
                    self.env.bind(param.name.value.clone(), param_type.clone());
                    param_types.push(param_type);
                }

                // Check body
                let mut body_type = Type::Void;
                for stmt in &func_def.body.statements {
                    body_type = self.check_statement(stmt)?;
                }

                self.env.pop_scope();

                let func_type = Type::function(param_types, body_type);
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

            Statement::While(while_stmt) => {
                let cond_type = self.infer_expression(&while_stmt.condition)?;
                if cond_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "While condition must be bool, got {}",
                        cond_type
                    )));
                }

                for stmt in &while_stmt.body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(Type::Void)
            }

            Statement::For(for_stmt) => {
                // Check init statement if present
                if let Some(init) = &for_stmt.init {
                    self.check_statement(init)?;
                }

                // Check condition
                let cond_type = self.infer_expression(&for_stmt.condition)?;
                if cond_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "For loop condition must be bool, got {}",
                        cond_type
                    )));
                }

                // Check update statement if present
                if let Some(update) = &for_stmt.update {
                    self.check_statement(update)?;
                }

                // Check body
                for stmt in &for_stmt.body.statements {
                    self.check_statement(stmt)?;
                }

                Ok(Type::Void)
            }

            Statement::ForIn(for_in_stmt) => {
                // Infer the type of the iterator expression
                let iterator_type = self.infer_expression(&for_in_stmt.iterator)?;

                // Verify that the iterator is an iterable type (Array, Range, etc.)
                match &iterator_type {
                    Type::Array(_) => {
                        // Valid array iterator
                    }
                    Type::Any => {
                        // Accept Any type (may be a range or other iterable)
                    }
                    _ => {
                        return Err(CompileError::Generic(format!(
                            "Cannot iterate over non-iterable type: {}",
                            iterator_type
                        )));
                    }
                }

                // Check body statements
                for stmt in &for_in_stmt.body.statements {
                    self.check_statement(stmt)?;
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

            Expression::Prefix(prefix) => {
                self.check_prefix_expression(prefix)
            }

            Expression::Infix(infix) => {
                self.check_infix_expression(infix)
            }

            Expression::FunctionCall(call) => {
                // Infer function type
                let func_type = self.infer_expression(&call.function)?;

                // Check if it's actually a function
                match &func_type {
                    Type::Function { params, return_type } => {
                        // Check argument count
                        if call.arguments.len() != params.len() {
                            return Err(CompileError::Generic(format!(
                                "Function expects {} arguments, got {}",
                                params.len(),
                                call.arguments.len()
                            )));
                        }

                        // Check argument types
                        for (i, (arg, expected_type)) in call.arguments.iter().zip(params.iter()).enumerate() {
                            let arg_type = self.infer_expression(arg)?;

                            // Try to unify the argument type with expected type
                            if let Err(e) = self.unify(&arg_type, expected_type) {
                                return Err(CompileError::Generic(format!(
                                    "Argument {} type mismatch: expected {}, got {}. {}",
                                    i + 1, expected_type, arg_type, e
                                )));
                            }
                        }

                        // Return the return type
                        Ok((**return_type).clone())
                    }
                    Type::Any => {
                        // If function type is Any (e.g., from external functions), skip checking
                        Ok(Type::Any)
                    }
                    _ => {
                        // Not a function type
                        Err(CompileError::Generic(format!(
                            "Cannot call non-function type: {}",
                            func_type
                        )))
                    }
                }
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

            Expression::ArrayLiteral(array_lit) => {
                if array_lit.elements.is_empty() {
                    // Empty array - unknown element type
                    Ok(Type::Array(Box::new(Type::Any)))
                } else {
                    // Infer type from first element
                    let first_type = self.infer_expression(&array_lit.elements[0])?;

                    // Check all elements have compatible types
                    for elem in &array_lit.elements[1..] {
                        let elem_type = self.infer_expression(elem)?;
                        self.unify(&elem_type, &first_type)?;
                    }

                    Ok(Type::Array(Box::new(first_type)))
                }
            }

            Expression::TupleLiteral(tuple_lit) => {
                // Infer type for each element
                let mut element_types = Vec::new();
                for elem in &tuple_lit.elements {
                    let elem_type = self.infer_expression(elem)?;
                    element_types.push(elem_type);
                }
                Ok(Type::Tuple(element_types))
            }

            Expression::StructLiteral(_) => {
                // For now, return Any for struct literals
                Ok(Type::Any)
            }

            Expression::FieldAccess(_) => {
                // For now, return Any for field access
                Ok(Type::Any)
            }

            Expression::IndexAccess(index_expr) => {
                // Process array and index expressions
                let array_type = self.infer_expression(&index_expr.array)?;
                let index_type = self.infer_expression(&index_expr.index)?;

                // Index must be an integer
                if index_type != Type::Int && index_type != Type::Any {
                    return Err(CompileError::Generic(format!(
                        "Array index must be an integer, got {}",
                        index_type
                    )));
                }

                // If array type is Array<T>, return T
                match array_type {
                    Type::Array(elem_type) => Ok(*elem_type),
                    Type::Any => Ok(Type::Any),
                    _ => Err(CompileError::Generic(format!(
                        "Cannot index into non-array type: {}",
                        array_type
                    ))),
                }
            }

            Expression::Match(_) => {
                // For now, return Any for match expressions
                Ok(Type::Any)
            }

            Expression::Borrow(borrow_expr) => {
                // Process the inner expression
                self.infer_expression(&borrow_expr.expression)?;
                // For now, return Any for borrow expressions
                Ok(Type::Any)
            }

            Expression::MutableBorrow(borrow_expr) => {
                // Process the inner expression
                self.infer_expression(&borrow_expr.expression)?;
                // For now, return Any for mutable borrow expressions
                Ok(Type::Any)
            }

            Expression::Dereference(deref_expr) => {
                // Process the inner expression
                self.infer_expression(&deref_expr.expression)?;
                // For now, return Any for dereference expressions
                Ok(Type::Any)
            }

            Expression::Range(_) => {
                // For now, return Any for range expressions
                Ok(Type::Any)
            }

            Expression::TryOperator(try_expr) => {
                // Process the inner expression recursively
                // In a full implementation, we would verify that the inner expression
                // returns a Result<T, E> type and extract the T type
                self.infer_expression(&try_expr.expression)
            }
        }
    }

    fn check_prefix_expression(&mut self, prefix: &PrefixExpression) -> Result<Type, CompileError> {
        let right_type = self.infer_expression(&prefix.right)?;
        let op = &prefix.operator.lexeme;

        match op.as_str() {
            "-" => {
                // Negation operator
                if !right_type.is_numeric() {
                    return Err(CompileError::Generic(format!(
                        "Cannot negate non-numeric type: {}",
                        right_type
                    )));
                }
                Ok(right_type)
            }
            "!" => {
                // Logical NOT operator
                if right_type != Type::Bool {
                    return Err(CompileError::Generic(format!(
                        "Logical NOT expects bool, got {}",
                        right_type
                    )));
                }
                Ok(Type::Bool)
            }
            _ => Err(CompileError::Generic(format!(
                "Unknown prefix operator: {}",
                op
            ))),
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
