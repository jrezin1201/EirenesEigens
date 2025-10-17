// RavensOne Type System
// Defines the type representation and type operations

use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    // Primitive types
    Int,
    Float,
    String,
    Bool,
    Void,
    Any,

    // Component types
    Component(Vec<Type>), // Component with prop types

    // Function types
    Function {
        params: Vec<Type>,
        return_type: Box<Type>,
    },

    // Collection types
    Array(Box<Type>),
    Tuple(Vec<Type>),

    // Generic types
    Generic(String), // Generic type variable (e.g., T, U)

    // Union types
    Union(Vec<Type>),

    // Optional type
    Option(Box<Type>),

    // Type variable (for inference)
    Var(usize),

    // Custom types
    Named(String),
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Int => write!(f, "int"),
            Type::Float => write!(f, "float"),
            Type::String => write!(f, "string"),
            Type::Bool => write!(f, "bool"),
            Type::Void => write!(f, "void"),
            Type::Any => write!(f, "any"),
            Type::Component(props) => {
                write!(f, "Component<")?;
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", prop)?;
                }
                write!(f, ">")
            }
            Type::Function { params, return_type } => {
                write!(f, "(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ") -> {}", return_type)
            }
            Type::Array(inner) => write!(f, "{}[]", inner),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                write!(f, ")")
            }
            Type::Generic(name) => write!(f, "{}", name),
            Type::Union(types) => {
                for (i, ty) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }
                    write!(f, "{}", ty)?;
                }
                Ok(())
            }
            Type::Option(inner) => write!(f, "{}?", inner),
            Type::Var(id) => write!(f, "τ{}", id),
            Type::Named(name) => write!(f, "{}", name),
        }
    }
}

impl Type {
    /// Check if this type is a primitive type
    pub fn is_primitive(&self) -> bool {
        matches!(self, Type::Int | Type::Float | Type::String | Type::Bool)
    }

    /// Check if this type is numeric
    pub fn is_numeric(&self) -> bool {
        matches!(self, Type::Int | Type::Float)
    }

    /// Check if two types are compatible (can be assigned)
    pub fn is_compatible_with(&self, other: &Type) -> bool {
        if self == other {
            return true;
        }

        match (self, other) {
            // Any type is compatible with everything
            (Type::Any, _) | (_, Type::Any) => true,

            // Numbers are inter-compatible
            (Type::Int, Type::Float) | (Type::Float, Type::Int) => true,

            // Optional types
            (Type::Option(inner), ty) | (ty, Type::Option(inner)) => {
                inner.as_ref().is_compatible_with(ty)
            }

            // Union types
            (Type::Union(types), ty) => types.iter().any(|t| t.is_compatible_with(ty)),
            (ty, Type::Union(types)) => types.iter().any(|t| ty.is_compatible_with(t)),

            // Arrays
            (Type::Array(a), Type::Array(b)) => a.is_compatible_with(b),

            _ => false,
        }
    }

    /// Get the return type of a function, if this is a function type
    pub fn get_return_type(&self) -> Option<&Type> {
        match self {
            Type::Function { return_type, .. } => Some(return_type),
            _ => None,
        }
    }

    /// Get the parameter types of a function, if this is a function type
    pub fn get_param_types(&self) -> Option<&Vec<Type>> {
        match self {
            Type::Function { params, .. } => Some(params),
            _ => None,
        }
    }

    /// Create a function type
    pub fn function(params: Vec<Type>, return_type: Type) -> Self {
        Type::Function {
            params,
            return_type: Box::new(return_type),
        }
    }

    /// Create an array type
    pub fn array(element_type: Type) -> Self {
        Type::Array(Box::new(element_type))
    }

    /// Create an optional type
    pub fn optional(inner_type: Type) -> Self {
        Type::Option(Box::new(inner_type))
    }
}

/// Type environment for tracking variable types in scopes
#[derive(Debug, Clone)]
pub struct TypeEnv {
    scopes: Vec<HashMap<String, Type>>,
    next_var_id: usize,
}

impl TypeEnv {
    pub fn new() -> Self {
        TypeEnv {
            scopes: vec![HashMap::new()],
            next_var_id: 0,
        }
    }

    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Add a variable binding in the current scope
    pub fn bind(&mut self, name: String, ty: Type) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Look up a variable's type
    pub fn lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Generate a fresh type variable
    pub fn fresh_var(&mut self) -> Type {
        let id = self.next_var_id;
        self.next_var_id += 1;
        Type::Var(id)
    }
}

impl Default for TypeEnv {
    fn default() -> Self {
        Self::new()
    }
}

/// Substitution for type variables
#[derive(Debug, Clone)]
pub struct Substitution {
    map: HashMap<usize, Type>,
}

impl Substitution {
    pub fn new() -> Self {
        Substitution {
            map: HashMap::new(),
        }
    }

    /// Add a substitution from type variable to type
    pub fn insert(&mut self, var: usize, ty: Type) {
        self.map.insert(var, ty);
    }

    /// Apply substitution to a type
    pub fn apply(&self, ty: &Type) -> Type {
        match ty {
            Type::Var(id) => {
                if let Some(substituted) = self.map.get(id) {
                    // Recursively apply substitution
                    self.apply(substituted)
                } else {
                    ty.clone()
                }
            }
            Type::Array(inner) => Type::Array(Box::new(self.apply(inner))),
            Type::Option(inner) => Type::Option(Box::new(self.apply(inner))),
            Type::Function { params, return_type } => Type::Function {
                params: params.iter().map(|p| self.apply(p)).collect(),
                return_type: Box::new(self.apply(return_type)),
            },
            Type::Tuple(types) => Type::Tuple(types.iter().map(|t| self.apply(t)).collect()),
            Type::Union(types) => Type::Union(types.iter().map(|t| self.apply(t)).collect()),
            _ => ty.clone(),
        }
    }

    /// Compose two substitutions
    pub fn compose(&self, other: &Substitution) -> Substitution {
        let mut result = Substitution::new();

        // Apply other to all types in self
        for (var, ty) in &self.map {
            result.insert(*var, other.apply(ty));
        }

        // Add mappings from other that aren't in self
        for (var, ty) in &other.map {
            if !result.map.contains_key(var) {
                result.insert(*var, ty.clone());
            }
        }

        result
    }
}

impl Default for Substitution {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_compatibility() {
        assert!(Type::Int.is_compatible_with(&Type::Int));
        assert!(Type::Int.is_compatible_with(&Type::Float));
        assert!(Type::Any.is_compatible_with(&Type::String));
        assert!(Type::Bool.is_compatible_with(&Type::Any));
    }

    #[test]
    fn test_type_env() {
        let mut env = TypeEnv::new();
        env.bind("x".to_string(), Type::Int);
        assert_eq!(env.lookup("x"), Some(&Type::Int));

        env.push_scope();
        env.bind("y".to_string(), Type::String);
        assert_eq!(env.lookup("y"), Some(&Type::String));
        assert_eq!(env.lookup("x"), Some(&Type::Int));

        env.pop_scope();
        assert_eq!(env.lookup("y"), None);
        assert_eq!(env.lookup("x"), Some(&Type::Int));
    }

    #[test]
    fn test_substitution() {
        let mut subst = Substitution::new();
        subst.insert(0, Type::Int);

        let ty = Type::Array(Box::new(Type::Var(0)));
        let result = subst.apply(&ty);
        assert_eq!(result, Type::Array(Box::new(Type::Int)));
    }
}
