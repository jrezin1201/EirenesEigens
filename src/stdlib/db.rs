/// Database ORM for RavensOne
///
/// Type-safe database queries with zero boilerplate
/// Supports SQLite, PostgreSQL, and edge databases (Cloudflare D1)

use std::collections::HashMap;
use std::marker::PhantomData;

/// Database column types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Integer,
    BigInt,
    Float,
    Text,
    Boolean,
    DateTime,
    Json,
}

impl ColumnType {
    pub fn to_sql(&self) -> &str {
        match self {
            ColumnType::Integer => "INTEGER",
            ColumnType::BigInt => "BIGINT",
            ColumnType::Float => "REAL",
            ColumnType::Text => "TEXT",
            ColumnType::Boolean => "BOOLEAN",
            ColumnType::DateTime => "DATETIME",
            ColumnType::Json => "JSON",
        }
    }
}

/// Database column definition
#[derive(Debug, Clone)]
pub struct Column {
    pub name: String,
    pub column_type: ColumnType,
    pub nullable: bool,
    pub primary_key: bool,
    pub auto_increment: bool,
    pub unique: bool,
    pub default_value: Option<String>,
}

impl Column {
    pub fn new(name: String, column_type: ColumnType) -> Self {
        Self {
            name,
            column_type,
            nullable: false,
            primary_key: false,
            auto_increment: false,
            unique: false,
            default_value: None,
        }
    }

    pub fn nullable(mut self) -> Self {
        self.nullable = true;
        self
    }

    pub fn primary_key(mut self) -> Self {
        self.primary_key = true;
        self
    }

    pub fn auto_increment(mut self) -> Self {
        self.auto_increment = true;
        self
    }

    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }

    pub fn default(mut self, value: String) -> Self {
        self.default_value = Some(value);
        self
    }

    pub fn to_sql(&self) -> String {
        let mut sql = format!("{} {}", self.name, self.column_type.to_sql());

        if self.primary_key {
            sql.push_str(" PRIMARY KEY");
        }

        if self.auto_increment {
            sql.push_str(" AUTOINCREMENT");
        }

        if self.unique && !self.primary_key {
            sql.push_str(" UNIQUE");
        }

        if !self.nullable && !self.primary_key {
            sql.push_str(" NOT NULL");
        }

        if let Some(ref default) = self.default_value {
            sql.push_str(&format!(" DEFAULT {}", default));
        }

        sql
    }
}

/// Database table schema
#[derive(Debug, Clone)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<Column>,
    pub indexes: Vec<String>,
}

impl TableSchema {
    pub fn new(name: String) -> Self {
        Self {
            name,
            columns: Vec::new(),
            indexes: Vec::new(),
        }
    }

    pub fn column(mut self, column: Column) -> Self {
        self.columns.push(column);
        self
    }

    pub fn index(mut self, index: String) -> Self {
        self.indexes.push(index);
        self
    }

    pub fn to_create_table_sql(&self) -> String {
        let columns_sql: Vec<String> = self.columns.iter()
            .map(|col| col.to_sql())
            .collect();

        format!(
            "CREATE TABLE IF NOT EXISTS {} (\n  {}\n);",
            self.name,
            columns_sql.join(",\n  ")
        )
    }
}

/// Query operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOp {
    Eq,
    NotEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Like,
    In,
    NotIn,
    IsNull,
    IsNotNull,
}

impl QueryOp {
    pub fn to_sql(&self) -> &str {
        match self {
            QueryOp::Eq => "=",
            QueryOp::NotEq => "!=",
            QueryOp::Lt => "<",
            QueryOp::Lte => "<=",
            QueryOp::Gt => ">",
            QueryOp::Gte => ">=",
            QueryOp::Like => "LIKE",
            QueryOp::In => "IN",
            QueryOp::NotIn => "NOT IN",
            QueryOp::IsNull => "IS NULL",
            QueryOp::IsNotNull => "IS NOT NULL",
        }
    }
}

/// Where clause condition
#[derive(Debug, Clone)]
pub struct WhereCondition {
    pub column: String,
    pub op: QueryOp,
    pub value: Option<String>,
}

impl WhereCondition {
    pub fn to_sql(&self) -> String {
        match &self.value {
            Some(val) => format!("{} {} {}", self.column, self.op.to_sql(), val),
            None => format!("{} {}", self.column, self.op.to_sql()),
        }
    }
}

/// Order direction
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderDirection {
    Asc,
    Desc,
}

impl OrderDirection {
    pub fn to_sql(&self) -> &str {
        match self {
            OrderDirection::Asc => "ASC",
            OrderDirection::Desc => "DESC",
        }
    }
}

/// Query builder for type-safe database queries
#[derive(Debug, Clone)]
pub struct QueryBuilder<T> {
    table: String,
    select_columns: Vec<String>,
    where_conditions: Vec<WhereCondition>,
    order_by: Vec<(String, OrderDirection)>,
    limit: Option<usize>,
    offset: Option<usize>,
    _phantom: PhantomData<T>,
}

impl<T> QueryBuilder<T> {
    pub fn new(table: String) -> Self {
        Self {
            table,
            select_columns: vec!["*".to_string()],
            where_conditions: Vec::new(),
            order_by: Vec::new(),
            limit: None,
            offset: None,
            _phantom: PhantomData,
        }
    }

    /// Select specific columns
    pub fn select(mut self, columns: Vec<String>) -> Self {
        self.select_columns = columns;
        self
    }

    /// Add WHERE condition
    pub fn where_eq(mut self, column: String, value: String) -> Self {
        self.where_conditions.push(WhereCondition {
            column,
            op: QueryOp::Eq,
            value: Some(value),
        });
        self
    }

    /// Add WHERE column > value
    pub fn where_gt(mut self, column: String, value: String) -> Self {
        self.where_conditions.push(WhereCondition {
            column,
            op: QueryOp::Gt,
            value: Some(value),
        });
        self
    }

    /// Add WHERE column < value
    pub fn where_lt(mut self, column: String, value: String) -> Self {
        self.where_conditions.push(WhereCondition {
            column,
            op: QueryOp::Lt,
            value: Some(value),
        });
        self
    }

    /// Add WHERE column LIKE pattern
    pub fn where_like(mut self, column: String, pattern: String) -> Self {
        self.where_conditions.push(WhereCondition {
            column,
            op: QueryOp::Like,
            value: Some(pattern),
        });
        self
    }

    /// Add WHERE column IS NULL
    pub fn where_null(mut self, column: String) -> Self {
        self.where_conditions.push(WhereCondition {
            column,
            op: QueryOp::IsNull,
            value: None,
        });
        self
    }

    /// Add ORDER BY clause
    pub fn order_by(mut self, column: String, direction: OrderDirection) -> Self {
        self.order_by.push((column, direction));
        self
    }

    /// Add LIMIT clause
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add OFFSET clause
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Build SQL SELECT query
    pub fn to_sql(&self) -> String {
        let mut sql = format!(
            "SELECT {} FROM {}",
            self.select_columns.join(", "),
            self.table
        );

        if !self.where_conditions.is_empty() {
            let conditions: Vec<String> = self.where_conditions.iter()
                .map(|c| c.to_sql())
                .collect();
            sql.push_str(&format!(" WHERE {}", conditions.join(" AND ")));
        }

        if !self.order_by.is_empty() {
            let orders: Vec<String> = self.order_by.iter()
                .map(|(col, dir)| format!("{} {}", col, dir.to_sql()))
                .collect();
            sql.push_str(&format!(" ORDER BY {}", orders.join(", ")));
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}

/// Database table interface
pub struct Table<T> {
    name: String,
    schema: TableSchema,
    _phantom: PhantomData<T>,
}

impl<T> Table<T> {
    pub fn new(name: String, schema: TableSchema) -> Self {
        Self {
            name,
            schema,
            _phantom: PhantomData,
        }
    }

    /// Get all records
    pub fn all(&self) -> QueryBuilder<T> {
        QueryBuilder::new(self.name.clone())
    }

    /// Find by ID
    pub fn find(&self, id: i32) -> QueryBuilder<T> {
        QueryBuilder::new(self.name.clone())
            .where_eq("id".to_string(), id.to_string())
            .limit(1)
    }

    /// Find by condition
    pub fn where_eq(&self, column: String, value: String) -> QueryBuilder<T> {
        QueryBuilder::new(self.name.clone())
            .where_eq(column, value)
    }

    /// Create new record
    pub fn create(&self, data: HashMap<String, String>) -> String {
        let columns: Vec<String> = data.keys().cloned().collect();
        let values: Vec<String> = data.values().cloned().collect();

        format!(
            "INSERT INTO {} ({}) VALUES ({})",
            self.name,
            columns.join(", "),
            values.join(", ")
        )
    }

    /// Update record
    pub fn update(&self, id: i32, data: HashMap<String, String>) -> String {
        let sets: Vec<String> = data.iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect();

        format!(
            "UPDATE {} SET {} WHERE id = {}",
            self.name,
            sets.join(", "),
            id
        )
    }

    /// Delete record
    pub fn delete(&self, id: i32) -> String {
        format!("DELETE FROM {} WHERE id = {}", self.name, id)
    }

    /// Get table schema
    pub fn get_schema(&self) -> &TableSchema {
        &self.schema
    }
}

/// Database connection
pub struct Database {
    tables: HashMap<String, TableSchema>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }

    /// Register a table schema
    pub fn register_table(&mut self, schema: TableSchema) {
        self.tables.insert(schema.name.clone(), schema);
    }

    /// Get table
    pub fn table<T>(&self, name: &str) -> Option<Table<T>> {
        self.tables.get(name).map(|schema| {
            Table::new(name.to_string(), schema.clone())
        })
    }

    /// Generate all migration SQL
    pub fn generate_migrations(&self) -> Vec<String> {
        self.tables.values()
            .map(|schema| schema.to_create_table_sql())
            .collect()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_column_sql() {
        let col = Column::new("id".to_string(), ColumnType::Integer)
            .primary_key()
            .auto_increment();

        assert_eq!(col.to_sql(), "id INTEGER PRIMARY KEY AUTOINCREMENT");
    }

    #[test]
    fn test_table_schema() {
        let schema = TableSchema::new("users".to_string())
            .column(
                Column::new("id".to_string(), ColumnType::Integer)
                    .primary_key()
                    .auto_increment()
            )
            .column(
                Column::new("name".to_string(), ColumnType::Text)
            )
            .column(
                Column::new("email".to_string(), ColumnType::Text)
                    .unique()
            );

        let sql = schema.to_create_table_sql();
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS users"));
        assert!(sql.contains("id INTEGER PRIMARY KEY AUTOINCREMENT"));
        assert!(sql.contains("name TEXT NOT NULL"));
        assert!(sql.contains("email TEXT UNIQUE NOT NULL"));
    }

    #[test]
    fn test_query_builder() {
        let query = QueryBuilder::<()>::new("users".to_string())
            .where_eq("active".to_string(), "true".to_string())
            .where_gt("age".to_string(), "18".to_string())
            .order_by("created_at".to_string(), OrderDirection::Desc)
            .limit(10);

        let sql = query.to_sql();
        assert_eq!(
            sql,
            "SELECT * FROM users WHERE active = true AND age > 18 ORDER BY created_at DESC LIMIT 10"
        );
    }

    #[test]
    fn test_table_operations() {
        let schema = TableSchema::new("posts".to_string())
            .column(
                Column::new("id".to_string(), ColumnType::Integer)
                    .primary_key()
                    .auto_increment()
            )
            .column(Column::new("title".to_string(), ColumnType::Text));

        let table = Table::<()>::new("posts".to_string(), schema);

        // Test find
        let query = table.find(1);
        assert_eq!(query.to_sql(), "SELECT * FROM posts WHERE id = 1 LIMIT 1");

        // Test create
        let mut data = HashMap::new();
        data.insert("title".to_string(), "'Hello'".to_string());
        let insert_sql = table.create(data);
        assert!(insert_sql.contains("INSERT INTO posts"));

        // Test update
        let mut data = HashMap::new();
        data.insert("title".to_string(), "'Updated'".to_string());
        let update_sql = table.update(1, data);
        assert_eq!(update_sql, "UPDATE posts SET title = 'Updated' WHERE id = 1");

        // Test delete
        let delete_sql = table.delete(1);
        assert_eq!(delete_sql, "DELETE FROM posts WHERE id = 1");
    }
}
