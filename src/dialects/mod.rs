#[cfg(feature = "mysql")]
use sqlx::mysql::MySqlRow;

#[cfg(feature = "postgres")]
use sqlx::pg::PgRow;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqliteRow;

mod mysql;
mod postgres;
mod sqlite;

use mysql::MySqlDialect;
use postgres::PostgresDialect;
use sqlite::SqliteDialect;

use crate::{
    filter::{FilterType, Filtered},
    helpers::{ColumnBindingKind, SqlBindQuery},
    schema::{ColumnInfo, Value},
};

/// Trait for database-specific SQL generation and binding behavior.
///
/// This trait is intended to hide most backend-specific details from higher-level
/// operations like `Insert`, `Update`, and filtering.
///
/// Only one backend feature (`mysql`, `postgres`, or `sqlite`) is expected to be
/// active at a time, so the associated `Row` type is specialized per dialect.
pub trait SqlDialect {
    /// The concrete row type for this dialect (e.g., `MySqlRow`, `PgRow`, `SqliteRow`).
    type Row;

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

    /// Extract a value from a backend-specific row into the generic [`Value`] enum.
    fn extract_column_value(
        &self,
        row: &Self::Row,
        column_name: &str,
        data_type: &str,
    ) -> Option<Value>;

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
    fn build_insert_sql<'a>(&self, table_name: &str, columns: &[ColumnInfo<'a>]) -> String {
        // INSERT INTO <table> (
        let mut sql = format!("INSERT INTO {} (", self.quote_identifier(table_name));

        // column list with proper quoting
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&self.quote_identifier(col.name));
        }

        sql.push_str(") VALUES (");

        // placeholders
        for (i, _col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&self.placeholder(i));
        }

        sql.push(')');

        sql
    }
}

/// A concrete trait-object type for the active backend.
/// The associated `Row` type is fixed by the selected feature.
#[cfg(feature = "mysql")]
pub type DynSqlDialect = dyn SqlDialect<Row = MySqlRow>;

#[cfg(all(not(feature = "mysql"), feature = "postgres"))]
pub type DynSqlDialect = dyn SqlDialect<Row = PgRow>;

#[cfg(all(not(feature = "mysql"), not(feature = "postgres"), feature = "sqlite"))]
pub type DynSqlDialect = dyn SqlDialect<Row = SqliteRow>;

/// Get the appropriate dialect for the current backend.
///
/// Only one of the branches below is compiled, depending on enabled features.
pub fn get_dialect() -> Box<DynSqlDialect> {
    #[cfg(feature = "mysql")]
    {
        return Box::new(MySqlDialect);
    }

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    {
        return Box::new(PostgresDialect);
    }

    #[cfg(all(not(feature = "mysql"), not(feature = "postgres"), feature = "sqlite"))]
    {
        return Box::new(SqliteDialect);
    }

    // If no supported backend feature is enabled, this will fail to compile,
    // which is desirable because the crate can't function without a backend.
}
