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
    Tuple(Vec<ResolvedType>),  // Tuple with element types
    Struct(String), // Struct type identified by name
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
            ResolvedType::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            ResolvedType::Struct(name) => write!(f, "{}", name),
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

/// Stores struct definitions for type checking
struct StructTable {
    structs: HashMap<String, HashMap<String, ResolvedType>>,  // struct_name -> (field_name -> field_type)
}

impl StructTable {
    fn new() -> Self {
        Self { structs: HashMap::new() }
    }

    fn define(&mut self, name: String, fields: HashMap<String, ResolvedType>) {
        self.structs.insert(name, fields);
    }

    fn get_field_type(&self, struct_name: &str, field_name: &str) -> Option<ResolvedType> {
        self.structs
            .get(struct_name)
            .and_then(|fields| fields.get(field_name))
            .cloned()
    }

    fn exists(&self, struct_name: &str) -> bool {
        self.structs.contains_key(struct_name)
    }
}

/// Tracks enum definitions for exhaustiveness checking
struct EnumTable {
    enums: HashMap<String, Vec<String>>,  // enum_name -> list of variant names
}

impl EnumTable {
    fn new() -> Self {
        Self { enums: HashMap::new() }
    }

    fn define(&mut self, name: String, variants: Vec<String>) {
        self.enums.insert(name, variants);
    }

    fn get_variants(&self, enum_name: &str) -> Option<&Vec<String>> {
        self.enums.get(enum_name)
    }

    fn exists(&self, enum_name: &str) -> bool {
        self.enums.contains_key(enum_name)
    }
}

/// Traverses the AST to perform type checking and symbol resolution.
pub struct SemanticAnalyzer {
    symbols: SymbolTable,
    structs: StructTable,  // Track struct definitions
    enums: EnumTable,  // Track enum definitions
    in_component: bool,  // Track if we're inside a component
    reactive_variables: HashSet<String>,  // Track reactive variable names
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbols: SymbolTable::new(),
            structs: StructTable::new(),
            enums: EnumTable::new(),
            in_component: false,
            reactive_variables: HashSet::new(),
        }
    }

    pub fn analyze_program(&mut self, program: &Program) -> Result<(), CompileError> {
        // First pass: collect struct and enum definitions
        for statement in &program.statements {
            match statement {
                Statement::Struct(struct_def) => {
                    self.register_struct(struct_def)?;
                }
                Statement::Enum(enum_def) => {
                    self.register_enum(enum_def)?;
                }
                _ => {}
            }
        }

        // Second pass: analyze statements
        for statement in &program.statements {
            self.analyze_statement(statement)?;
        }
        Ok(())
    }

    fn register_struct(&mut self, struct_def: &StructDefinition) -> Result<(), CompileError> {
        let mut field_types = HashMap::new();
        for (field_name, field_type_expr) in &struct_def.fields {
            // Convert TypeExpression to ResolvedType
            let field_type = self.type_expression_to_resolved_type(field_type_expr);
            field_types.insert(field_name.value.clone(), field_type);
        }
        self.structs.define(struct_def.name.value.clone(), field_types);
        Ok(())
    }

    fn register_enum(&mut self, enum_def: &EnumDefinition) -> Result<(), CompileError> {
        let variant_names: Vec<String> = enum_def.variants.iter()
            .map(|v| v.name.value.clone())
            .collect();
        self.enums.define(enum_def.name.value.clone(), variant_names);
        Ok(())
    }

    fn type_expression_to_resolved_type(&self, type_expr: &TypeExpression) -> ResolvedType {
        match type_expr {
            TypeExpression::Named(ident) => {
                match ident.value.as_str() {
                    "i32" => ResolvedType::Integer,
                    "f64" => ResolvedType::Float,
                    "bool" => ResolvedType::Bool,
                    "string" => ResolvedType::String,
                    _ => {
                        // Check if it's a struct type
                        if self.structs.exists(&ident.value) {
                            ResolvedType::Struct(ident.value.clone())
                        } else {
                            ResolvedType::Unknown
                        }
                    }
                }
            }
            TypeExpression::Generic(ident, args) => {
                // Handle generic types like Array<T>
                if ident.value == "Array" && !args.is_empty() {
                    let inner_type = self.type_expression_to_resolved_type(&args[0]);
                    ResolvedType::Array(Box::new(inner_type))
                } else {
                    ResolvedType::Unknown
                }
            }
            TypeExpression::Tuple(types) => {
                // Convert tuple type expression to resolved tuple type
                let resolved_types: Vec<ResolvedType> = types.iter()
                    .map(|t| self.type_expression_to_resolved_type(t))
                    .collect();
                ResolvedType::Tuple(resolved_types)
            }
            TypeExpression::Reference(_inner) => {
                // For now, return Unknown for reference types
                ResolvedType::Unknown
            }
            TypeExpression::MutableReference(_inner) => {
                // For now, return Unknown for mutable reference types
                ResolvedType::Unknown
            }
            TypeExpression::Slice(inner) => {
                // Slices are array views - recursively process the inner type
                // Return ResolvedType::Array with the inner type
                let inner_type = self.type_expression_to_resolved_type(inner);
                ResolvedType::Array(Box::new(inner_type))
            }
        }
    }

    fn analyze_statement(&mut self, stmt: &Statement) -> Result<ResolvedType, CompileError> {
        match stmt {
            Statement::Use(_) => Ok(ResolvedType::Unit),
            Statement::Let(let_stmt) => self.analyze_let_statement(let_stmt),
            Statement::Assignment(assign_stmt) => {
                // Check that variable exists
                if self.symbols.lookup(&assign_stmt.target.value).is_none() {
                    return Err(CompileError::Generic(format!(
                        "Cannot assign to undefined variable '{}'",
                        assign_stmt.target.value
                    )));
                }

                // Analyze the value expression
                self.analyze_expression(&assign_stmt.value)?;

                Ok(ResolvedType::Unit)
            }
            Statement::Return(return_stmt) => self.analyze_return_statement(return_stmt),
            Statement::Expression(expr) => self.analyze_expression(expr),
            Statement::If(if_stmt) => self.analyze_if_statement(if_stmt),
            Statement::While(while_stmt) => self.analyze_while_statement(while_stmt),
            Statement::For(for_stmt) => self.analyze_for_statement(for_stmt),
            Statement::ForIn(for_in_stmt) => self.analyze_for_in_statement(for_in_stmt),
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
            Statement::Enum(_) => Ok(ResolvedType::Unit),
            Statement::ImplBlock(_) => Ok(ResolvedType::Unit),
            Statement::Trait(_) => Ok(ResolvedType::Unit),
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

    fn analyze_while_statement(&mut self, stmt: &WhileStatement) -> Result<ResolvedType, CompileError> {
        let cond_type = self.analyze_expression(&stmt.condition)?;
        if cond_type != ResolvedType::Bool && cond_type != ResolvedType::Integer {
            return Err(CompileError::Generic(format!(
                "While condition must be bool or integer, got '{}'",
                cond_type
            )));
        }

        // Analyze loop body
        for s in &stmt.body.statements {
            self.analyze_statement(s)?;
        }

        Ok(ResolvedType::Unit)
    }

    fn analyze_for_statement(&mut self, stmt: &ForStatement) -> Result<ResolvedType, CompileError> {
        // Analyze init statement if present
        if let Some(init) = &stmt.init {
            self.analyze_statement(init)?;
        }

        // Analyze condition
        let cond_type = self.analyze_expression(&stmt.condition)?;
        if cond_type != ResolvedType::Bool && cond_type != ResolvedType::Integer {
            return Err(CompileError::Generic(format!(
                "For loop condition must be bool or integer, got '{}'",
                cond_type
            )));
        }

        // Analyze update statement if present
        if let Some(update) = &stmt.update {
            self.analyze_statement(update)?;
        }

        // Analyze loop body
        for s in &stmt.body.statements {
            self.analyze_statement(s)?;
        }

        Ok(ResolvedType::Unit)
    }

    fn analyze_for_in_statement(&mut self, stmt: &ForInStatement) -> Result<ResolvedType, CompileError> {
        // Analyze the iterator expression
        let iterator_type = self.analyze_expression(&stmt.iterator)?;

        // Verify that the iterator type is iterable (Array, Range, etc.)
        // For now, we accept Array types and Unknown types
        match &iterator_type {
            ResolvedType::Array(_) => {
                // Valid iterator type
            }
            ResolvedType::Unknown => {
                // Accept unknown types (may be a range or other iterable)
            }
            _ => {
                return Err(CompileError::Generic(format!(
                    "Cannot iterate over non-iterable type '{}'",
                    iterator_type
                )));
            }
        }

        // Analyze body statements
        for s in &stmt.body.statements {
            self.analyze_statement(s)?;
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
            Expression::Prefix(prefix_expr) => {
                self.analyze_expression(&prefix_expr.right)?;
                Ok(ResolvedType::Integer)
            }
            Expression::Infix(infix_expr) => self.analyze_infix_expression(infix_expr),
            Expression::ArrayLiteral(array_lit) => {
                if array_lit.elements.is_empty() {
                    // Empty array - we don't know the element type yet
                    Ok(ResolvedType::Array(Box::new(ResolvedType::Unknown)))
                } else {
                    // Infer type from first element
                    let first_type = self.analyze_expression(&array_lit.elements[0])?;
                    // TODO: Verify all elements have the same type
                    for elem in &array_lit.elements[1..] {
                        self.analyze_expression(elem)?;
                    }
                    Ok(ResolvedType::Array(Box::new(first_type)))
                }
            }
            Expression::TupleLiteral(tuple_lit) => {
                // Infer type for each element
                let mut element_types = Vec::new();
                for elem in &tuple_lit.elements {
                    let elem_type = self.analyze_expression(elem)?;
                    element_types.push(elem_type);
                }
                Ok(ResolvedType::Tuple(element_types))
            }
            Expression::StructLiteral(struct_lit) => {
                // Check if struct exists
                if !self.structs.exists(&struct_lit.name.value) {
                    return Err(CompileError::Generic(format!(
                        "Unknown struct type: '{}'",
                        struct_lit.name.value
                    )));
                }

                // Analyze all field values and check types
                for (field_name, field_value) in &struct_lit.fields {
                    let value_type = self.analyze_expression(field_value)?;

                    // Look up expected field type
                    if let Some(expected_type) = self.structs.get_field_type(&struct_lit.name.value, &field_name.value) {
                        // TODO: Add type compatibility checking here
                        // For now, just accept any type
                    } else {
                        return Err(CompileError::Generic(format!(
                            "Struct '{}' has no field named '{}'",
                            struct_lit.name.value,
                            field_name.value
                        )));
                    }
                }

                // Return the struct type
                Ok(ResolvedType::Struct(struct_lit.name.value.clone()))
            }
            Expression::FieldAccess(field_access) => {
                // Analyze the object expression to get its type
                let object_type = self.analyze_expression(&field_access.object)?;

                // If it's a struct type, look up the field
                match object_type {
                    ResolvedType::Struct(struct_name) => {
                        if let Some(field_type) = self.structs.get_field_type(&struct_name, &field_access.field.value) {
                            Ok(field_type)
                        } else {
                            Err(CompileError::Generic(format!(
                                "Struct '{}' has no field named '{}'",
                                struct_name,
                                field_access.field.value
                            )))
                        }
                    }
                    _ => {
                        // For non-struct types, return Unknown for now
                        Ok(ResolvedType::Unknown)
                    }
                }
            }
            Expression::IndexAccess(index_expr) => {
                // Analyze the array expression
                let array_type = self.analyze_expression(&index_expr.array)?;

                // Analyze the index expression (should be integer)
                let index_type = self.analyze_expression(&index_expr.index)?;
                if index_type != ResolvedType::Integer && index_type != ResolvedType::Unknown {
                    return Err(CompileError::Generic(format!(
                        "Array index must be an integer, got '{}'",
                        index_type
                    )));
                }

                // Return the element type if it's an array
                match array_type {
                    ResolvedType::Array(element_type) => Ok(*element_type),
                    ResolvedType::Unknown => Ok(ResolvedType::Unknown),
                    _ => Err(CompileError::Generic(format!(
                        "Cannot index into non-array type '{}'",
                        array_type
                    ))),
                }
            }
            Expression::Match(match_expr) => {
                // Analyze the scrutinee to get its type
                let scrutinee_type = self.analyze_expression(&match_expr.scrutinee)?;

                // Check exhaustiveness if matching on an enum
                self.check_match_exhaustiveness(match_expr, &scrutinee_type)?;

                // Analyze all match arms and infer the result type
                if match_expr.arms.is_empty() {
                    return Ok(ResolvedType::Unit);
                }

                // Get the type of the first arm's body
                let first_arm_type = self.analyze_expression(&match_expr.arms[0].body)?;

                // Verify all other arms return the same type
                for arm in &match_expr.arms[1..] {
                    let arm_type = self.analyze_expression(&arm.body)?;

                    // Check type compatibility
                    if !self.types_compatible(&arm_type, &first_arm_type) {
                        return Err(CompileError::Generic(format!(
                            "Match arms have incompatible types: expected '{}', found '{}'",
                            first_arm_type, arm_type
                        )));
                    }
                }

                // All arms have compatible types, return the common type
                Ok(first_arm_type)
            }
            Expression::JsxElement(_) => Ok(ResolvedType::VNode),
            Expression::FunctionCall(_) => Ok(ResolvedType::Unknown),
            Expression::Lambda(_) => Ok(ResolvedType::Unknown),
            Expression::Borrow(borrow_expr) => {
                self.analyze_expression(&borrow_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::MutableBorrow(borrow_expr) => {
                self.analyze_expression(&borrow_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::Dereference(deref_expr) => {
                self.analyze_expression(&deref_expr.expression)?;
                Ok(ResolvedType::Unknown)
            }
            Expression::Range(_range_expr) => {
                // Range expressions are placeholders for now
                // In a full implementation, we'd analyze the start and end expressions
                Ok(ResolvedType::Unknown)
            }
            Expression::TryOperator(try_expr) => {
                // Analyze the inner expression and return its type
                // In a full implementation, we would verify that the inner expression
                // returns a Result<T, E> type and extract the T type
                self.analyze_expression(&try_expr.expression)
            }
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

    fn types_compatible(&self, type1: &ResolvedType, type2: &ResolvedType) -> bool {
        // For now, require exact type equality
        // In a full implementation, we'd handle:
        // - Subtyping
        // - Type coercion (e.g., i32 -> f64)
        // - Unknown types (which are compatible with anything)

        match (type1, type2) {
            // Unknown types are compatible with anything
            (ResolvedType::Unknown, _) | (_, ResolvedType::Unknown) => true,
            // Complex types are compatible with anything (for now)
            (ResolvedType::ComplexType, _) | (_, ResolvedType::ComplexType) => true,
            // Otherwise, require exact equality
            _ => type1 == type2,
        }
    }

    fn check_match_exhaustiveness(&self, match_expr: &MatchExpression, _scrutinee_type: &ResolvedType) -> Result<(), CompileError> {
        // Collect all patterns from match arms
        let mut covered_variants: HashSet<String> = HashSet::new();
        let mut has_wildcard = false;

        for arm in &match_expr.arms {
            match &arm.pattern {
                Pattern::Wildcard => {
                    has_wildcard = true;
                }
                Pattern::EnumVariant { name, .. } => {
                    // Extract the variant name (could be "Color::Red" format)
                    let variant_name = if name.value.contains("::") {
                        // Split "Color::Red" into ["Color", "Red"]
                        let parts: Vec<&str> = name.value.split("::").collect();
                        if parts.len() == 2 {
                            parts[1].to_string()
                        } else {
                            name.value.clone()
                        }
                    } else {
                        name.value.clone()
                    };
                    covered_variants.insert(variant_name);
                }
                Pattern::Identifier(_) => {
                    // Identifier patterns act like wildcards
                    has_wildcard = true;
                }
                Pattern::Literal(_) => {
                    // Literals don't contribute to enum exhaustiveness
                }
            }
        }

        // If we have a wildcard pattern, match is exhaustive
        if has_wildcard {
            return Ok(());
        }

        // Check if we're matching on an enum
        // Try to extract enum name from first variant pattern
        for arm in &match_expr.arms {
            if let Pattern::EnumVariant { name, .. } = &arm.pattern {
                if name.value.contains("::") {
                    let parts: Vec<&str> = name.value.split("::").collect();
                    if parts.len() == 2 {
                        let enum_name = parts[0];

                        // Check if this enum exists
                        if let Some(variants) = self.enums.get_variants(enum_name) {
                            // Check if all variants are covered
                            let uncovered: Vec<String> = variants.iter()
                                .filter(|v| !covered_variants.contains(*v))
                                .cloned()
                                .collect();

                            if !uncovered.is_empty() {
                                return Err(CompileError::Generic(format!(
                                    "Match expression is not exhaustive. Missing variants: {}",
                                    uncovered.join(", ")
                                )));
                            }
                        }
                        break;
                    }
                }
            }
        }

        Ok(())
    }
}