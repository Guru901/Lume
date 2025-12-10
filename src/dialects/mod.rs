#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
use mysql::MySqlDialect;
#[cfg(feature = "postgres")]
use postgres::PostgresDialect;
#[cfg(feature = "sqlite")]
use sqlite::SqliteDialect;

use crate::{
    filter::FilterType,
    helpers::{ColumnBindingKind, SqlBindQuery},
    schema::ColumnInfo,
};

/// Trait for database-specific SQL generation and binding behavior.
///
/// This trait is intended to hide most backend-specific details from higher-level
/// operations like `Insert`, `Update`, and filtering.
///
/// Only one backend feature (`mysql`, `postgres`, or `sqlite`) is expected to be
/// active at a time, so the associated `Row` type is specialized per dialect.
pub trait SqlDialect {
    /// Quote an identifier (table name, column name) according to backend rules.
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Generate a placeholder for the given index (0-based).
    ///
    /// - MySQL / SQLite: always `"?"` (index is ignored)
    /// - Postgres: `"$1"`, `"$2"`, ... (1-based)
    fn placeholder(&self, index: usize) -> String;

    /// Adapt SQL syntax for backend-specific requirements (e.g., DDL rewrites).
    fn adapt_sql(&self, sql: String) -> String;

    /// Add or adapt a `RETURNING` clause.
    ///
    /// For:
    /// - Postgres / SQLite: appends `RETURNING col1, col2, ...;`
    /// - MySQL: usually a no-op (returns the same `sql`)
    fn returning_sql(&self, sql: String, returning: &Vec<&'static str>) -> String;

    /// Build a simple filter expression when no specialized behavior is needed.
    ///
    /// The default implementation can be overridden per dialect if needed.
    ///
    /// Example (MySQL / SQLite):
    /// `table.column = ?`
    ///
    /// Example (Postgres):
    /// `table.column = $1`
    fn build_filter_expr_fallback(
        &self,
        col1: &(String, String),
        filter: &FilterType,
        idx: usize,
    ) -> String;

    /// Bind a `NULL` of the appropriate Rust type for this dialect.
    fn bind_null<'q>(&self, query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q>;

    /// Build a complete parameterized `INSERT` SQL statement:
    ///
    /// `INSERT INTO <table> (<col1>, <col2>, ...) VALUES (<placeholders...>)`
    ///
    /// This centralizes placeholder style and identifier quoting for all backends.
    ///
    /// Intended to replace `Insert::insert_sql` so that the insert operation
    /// no longer needs any `#[cfg(feature = "...")]` logic for SQL construction.
    fn insert_sql(&self, sql: String, columns: &Vec<ColumnInfo>) -> String;

    // fn returning() -> String;
}

/// Get the appropriate dialect for the current backend.
///
/// Only one of the branches below is compiled, depending on enabled features.
pub fn get_dialect() -> Box<dyn SqlDialect> {
    #[cfg(feature = "mysql")]
    {
        return Box::new(MySqlDialect);
    }

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    {
        return Box::new(PostgresDialect);
    }

    #[cfg(all(not(feature = "mysql"), not(feature = "postgres"), feature = "sqlite"))]
    return Box::new(SqliteDialect);

    #[cfg(all(
        not(feature = "mysql"),
        not(feature = "postgres"),
        not(feature = "sqlite")
    ))]
    compile_error!(
        "At least one database backend feature (mysql, postgres, sqlite) must be enabled"
    );
}
