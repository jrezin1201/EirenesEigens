/// RavensOne Standard Library
///
/// Provides batteries-included functionality for building full-stack applications.

pub mod reactive;
pub mod collections;
pub mod http;
pub mod db;
pub mod auth;
pub mod option;
pub mod result;
pub mod iterator;
pub mod vec;
pub mod json;
pub mod time;
pub mod hashmap;
pub mod string;
pub mod fs;

// Re-export commonly used items
pub use reactive::{Signal, Computed, Effect};
pub use collections::{RArray, RMap};
pub use http::{HttpClient, HttpRequest, HttpResponse, HttpMethod};
pub use db::{Database, Table, TableSchema, Column, ColumnType, QueryBuilder, OrderDirection};
pub use auth::{User, SafeUser, UserRole, AuthToken, Session, AuthService, Claims};
