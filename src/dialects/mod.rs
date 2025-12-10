#[cfg(feature = "mysql")]
use sqlx::mysql::MySqlRow;

#[cfg(feature = "postgres")]
use sqlx::pg::PgRow;

#[cfg(feature = "sqlite")]
use sqlx::sqlite::SqliteRow;

use crate::{
    filter::FilterType,
    helpers::{ColumnBindingKind, SqlBindQuery},
    schema::Value,
};

/// Trait for database-specific SQL generation
pub trait SqlDialect {
    /// Quote an identifier (table name, column name) according to backend rules
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Generate a placeholder for the given index (0-based)
    fn placeholder(&self, index: usize) -> String;

    /// Adapt SQL syntax for backend-specific requirements
    fn adapt_sql(&self, sql: String) -> String;

    fn returning_sql(&self, sql: String, returning: &Vec<&'static str>) -> String;

    fn extract_column_value(
        &self,
        row: &MySqlRow,
        column_name: &str,
        data_type: &str,
    ) -> Option<Value>;

    fn build_filter_expr_fallback(
        &self,
        col1: &(String, String),
        filter: &FilterType,
        idx: usize,
    ) -> String;

    fn bind_null<'q>(&self, query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q>;
}

// MySQL Implementation
#[allow(unused)]
pub struct MySqlDialect;

impl SqlDialect for MySqlDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // MySQL uses backticks and escapes existing backticks by doubling
        format!("`{}`", identifier.replace('`', "``"))
    }

    fn placeholder(&self, _index: usize) -> String {
        // MySQL uses ? for all placeholders
        "?".to_string()
    }

    fn adapt_sql(&self, sql: String) -> String {
        // MySQL-specific transformations (if needed)
        sql
    }
    fn returning_sql(&self, sql: String, _returning: &Vec<&'static str>) -> String {
        sql
    }

    fn extract_column_value(
        &self,
        row: &MySqlRow,
        column_name: &str,
        data_type: &str,
    ) -> Option<Value> {
        use sqlx::Row as _;
        match data_type {
            "TEXT" => {
                // Try to get as string first
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
            "TINYINT" => {
                if let Ok(val) = row.try_get::<i8, _>(column_name) {
                    Some(Value::Int8(val))
                } else if let Ok(val) = row.try_get::<Option<i8>, _>(column_name) {
                    val.map(Value::Int8)
                } else {
                    None
                }
            }
            "SMALLINT" => {
                if let Ok(val) = row.try_get::<i16, _>(column_name) {
                    Some(Value::Int16(val))
                } else if let Ok(val) = row.try_get::<Option<i16>, _>(column_name) {
                    val.map(Value::Int16)
                } else {
                    None
                }
            }
            "INTEGER" => {
                if let Ok(val) = row.try_get::<i32, _>(column_name) {
                    Some(Value::Int32(val))
                } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                    val.map(Value::Int32)
                } else {
                    None
                }
            }
            "BIGINT" => {
                if let Ok(val) = row.try_get::<i64, _>(column_name) {
                    Some(Value::Int64(val))
                } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                    val.map(Value::Int64)
                } else {
                    None
                }
            }
            "TINYINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u8, _>(column_name) {
                    Some(Value::UInt8(val))
                } else if let Ok(val) = row.try_get::<Option<u8>, _>(column_name) {
                    val.map(Value::UInt8)
                } else {
                    None
                }
            }
            "SMALLINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u16, _>(column_name) {
                    Some(Value::UInt16(val))
                } else if let Ok(val) = row.try_get::<Option<u16>, _>(column_name) {
                    val.map(Value::UInt16)
                } else {
                    None
                }
            }
            "INTEGER UNSIGNED" => {
                if let Ok(val) = row.try_get::<u32, _>(column_name) {
                    Some(Value::UInt32(val))
                } else if let Ok(val) = row.try_get::<Option<u32>, _>(column_name) {
                    val.map(Value::UInt32)
                } else {
                    None
                }
            }
            "BIGINT UNSIGNED" => {
                if let Ok(val) = row.try_get::<u64, _>(column_name) {
                    Some(Value::UInt64(val))
                } else if let Ok(val) = row.try_get::<Option<u64>, _>(column_name) {
                    val.map(Value::UInt64)
                } else {
                    None
                }
            }
            "FLOAT" => {
                if let Ok(val) = row.try_get::<f32, _>(column_name) {
                    Some(Value::Float32(val))
                } else if let Ok(val) = row.try_get::<Option<f32>, _>(column_name) {
                    val.map(Value::Float32)
                } else {
                    None
                }
            }
            "REAL" | "DOUBLE PRECISION" | "DOUBLE" => {
                if let Ok(val) = row.try_get::<f64, _>(column_name) {
                    Some(Value::Float64(val))
                } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                    val.map(Value::Float64)
                } else {
                    None
                }
            }
            "BOOLEAN" => {
                if let Ok(val) = row.try_get::<bool, _>(column_name) {
                    Some(Value::Bool(val))
                } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name) {
                    val.map(Value::Bool)
                } else {
                    None
                }
            }
            _ => {
                // Fallback: try to get as string
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
        }
    }

    fn build_filter_expr_fallback(
        &self,
        col1: &(String, String),
        filter: &FilterType,
        _idx: usize,
    ) -> String {
        format!("{}.{} {} ?", col1.0, col1.1, filter.to_sql())
    }

    fn bind_null<'q>(&self, query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q> {
        match kind {
            ColumnBindingKind::Varchar | ColumnBindingKind::Text | ColumnBindingKind::Unknown => {
                query.bind(None::<&str>)
            }
            ColumnBindingKind::TinyInt => query.bind(None::<i8>),
            ColumnBindingKind::SmallInt => query.bind(None::<i16>),
            ColumnBindingKind::Integer => query.bind(None::<i32>),
            ColumnBindingKind::BigInt => query.bind(None::<i64>),
            ColumnBindingKind::TinyIntUnsigned => query.bind(None::<u8>),
            ColumnBindingKind::SmallIntUnsigned => query.bind(None::<u16>),

            ColumnBindingKind::IntegerUnsigned => query.bind(None::<u32>),
            ColumnBindingKind::BigIntUnsigned => query.bind(None::<u64>),
            ColumnBindingKind::Float => query.bind(None::<f32>),
            ColumnBindingKind::Double => query.bind(None::<f64>),
            ColumnBindingKind::Boolean => query.bind(None::<bool>),
        }
    }
}

// PostgreSQL Implementation
#[allow(unused)]
pub struct PostgresDialect;

impl SqlDialect for PostgresDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // Postgres uses double quotes and escapes by doubling
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn placeholder(&self, index: usize) -> String {
        // Postgres uses $1, $2, $3, etc. (1-indexed!)
        format!("${}", index + 1)
    }

    fn adapt_sql(&self, sql: String) -> String {
        sql.replace("AUTO_INCREMENT", "GENERATED ALWAYS AS IDENTITY")
            .replace("DEFAULT (UUID())", "DEFAULT gen_random_uuid()")
    }

    fn returning_sql(&self, mut sql: String, returning: &Vec<&'static str>) -> String {
        if returning.is_empty() {
            return sql;
        }

        sql.push_str(" RETURNING ");
        for (i, col) in returning.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col);
        }
        sql.push_str(";");
        sql
    }

    fn extract_column_value(
        &self,
        row: &MySqlRow,
        column_name: &str,
        data_type: &str,
    ) -> Option<Value> {
        use sqlx::Row as _;
        match data_type {
            "TEXT" => {
                // Try to get as string first
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
            "SMALLINT" => {
                if let Ok(val) = row.try_get::<i16, _>(column_name) {
                    Some(Value::Int16(val))
                } else if let Ok(val) = row.try_get::<Option<i16>, _>(column_name) {
                    val.map(Value::Int16)
                } else {
                    None
                }
            }
            "INTEGER" => {
                if let Ok(val) = row.try_get::<i32, _>(column_name) {
                    Some(Value::Int32(val))
                } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                    val.map(Value::Int32)
                } else {
                    None
                }
            }
            "BIGINT" => {
                if let Ok(val) = row.try_get::<i64, _>(column_name) {
                    Some(Value::Int64(val))
                } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                    val.map(Value::Int64)
                } else {
                    None
                }
            }
            "FLOAT" => {
                if let Ok(val) = row.try_get::<f32, _>(column_name) {
                    Some(Value::Float32(val))
                } else if let Ok(val) = row.try_get::<Option<f32>, _>(column_name) {
                    val.map(Value::Float32)
                } else {
                    None
                }
            }
            "REAL" | "DOUBLE PRECISION" | "DOUBLE" => {
                if let Ok(val) = row.try_get::<f64, _>(column_name) {
                    Some(Value::Float64(val))
                } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                    val.map(Value::Float64)
                } else {
                    None
                }
            }
            "BOOLEAN" => {
                if let Ok(val) = row.try_get::<bool, _>(column_name) {
                    Some(Value::Bool(val))
                } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name) {
                    val.map(Value::Bool)
                } else {
                    None
                }
            }
            _ => {
                // Fallback: try to get as string
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
        }
    }

    fn build_filter_expr_fallback(
        &self,
        col1: &(String, String),
        filter: &FilterType,
        idx: usize,
    ) -> String {
        format!("{}.{} {} ${}", col1.0, col1.1, filter.to_sql(), idx)
    }

    fn bind_null<'q>(&self, query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q> {
        match kind {
            ColumnBindingKind::Varchar | ColumnBindingKind::Text | ColumnBindingKind::Unknown => {
                query.bind(None::<&str>)
            }
            ColumnBindingKind::TinyInt | ColumnBindingKind::TinyIntUnsigned => {
                query.bind(None::<i16>)
            }
            ColumnBindingKind::SmallInt => query.bind(None::<i16>),
            ColumnBindingKind::SmallIntUnsigned => query.bind(None::<i32>),
            ColumnBindingKind::Integer => query.bind(None::<i32>),
            ColumnBindingKind::IntegerUnsigned => query.bind(None::<i64>),
            ColumnBindingKind::BigInt | ColumnBindingKind::BigIntUnsigned => {
                query.bind(None::<i64>)
            }
            ColumnBindingKind::Float => query.bind(None::<f32>),
            ColumnBindingKind::Double => query.bind(None::<f64>),
            ColumnBindingKind::Boolean => query.bind(None::<bool>),
        }
    }
}

// SQLite Implementation
#[allow(unused)]
pub struct SqliteDialect;

impl SqlDialect for SqliteDialect {
    fn quote_identifier(&self, identifier: &str) -> String {
        // SQLite uses double quotes like Postgres
        format!("\"{}\"", identifier.replace('"', "\"\""))
    }

    fn placeholder(&self, _index: usize) -> String {
        // SQLite uses ? like MySQL
        "?".to_string()
    }

    fn adapt_sql(&self, sql: String) -> String {
        sql.replace("DEFAULT (UUID())", "DEFAULT (lower(hex(randomblob(16))))")
            .replace("DATETIME", "TEXT")
            .replace("CURRENT_TIMESTAMP", "(datetime('now'))")
            // Remove AUTO_INCREMENT
            .replace(" AUTO_INCREMENT", "")
            .replace("AUTO_INCREMENT ", "")
    }

    fn returning_sql(&self, mut sql: String, returning: &Vec<&'static str>) -> String {
        if returning.is_empty() {
            return sql;
        }

        sql.push_str(" RETURNING ");
        for (i, col) in returning.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(col);
        }
        sql.push_str(";");
        sql
    }

    fn extract_column_value(
        &self,
        row: &MySqlRow,
        column_name: &str,
        data_type: &str,
    ) -> Option<Value> {
        use sqlx::Row as _;
        match data_type {
            "TEXT" => {
                // Try to get as string first
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
            "SMALLINT" => {
                if let Ok(val) = row.try_get::<i16, _>(column_name) {
                    Some(Value::Int16(val))
                } else if let Ok(val) = row.try_get::<Option<i16>, _>(column_name) {
                    val.map(Value::Int16)
                } else {
                    None
                }
            }
            "INTEGER" => {
                if let Ok(val) = row.try_get::<i32, _>(column_name) {
                    Some(Value::Int32(val))
                } else if let Ok(val) = row.try_get::<Option<i32>, _>(column_name) {
                    val.map(Value::Int32)
                } else {
                    None
                }
            }
            "BIGINT" => {
                if let Ok(val) = row.try_get::<i64, _>(column_name) {
                    Some(Value::Int64(val))
                } else if let Ok(val) = row.try_get::<Option<i64>, _>(column_name) {
                    val.map(Value::Int64)
                } else {
                    None
                }
            }
            "FLOAT" => {
                if let Ok(val) = row.try_get::<f32, _>(column_name) {
                    Some(Value::Float32(val))
                } else if let Ok(val) = row.try_get::<Option<f32>, _>(column_name) {
                    val.map(Value::Float32)
                } else {
                    None
                }
            }
            "REAL" | "DOUBLE PRECISION" | "DOUBLE" => {
                if let Ok(val) = row.try_get::<f64, _>(column_name) {
                    Some(Value::Float64(val))
                } else if let Ok(val) = row.try_get::<Option<f64>, _>(column_name) {
                    val.map(Value::Float64)
                } else {
                    None
                }
            }
            "BOOLEAN" => {
                if let Ok(val) = row.try_get::<bool, _>(column_name) {
                    Some(Value::Bool(val))
                } else if let Ok(val) = row.try_get::<Option<bool>, _>(column_name) {
                    val.map(Value::Bool)
                } else {
                    None
                }
            }
            _ => {
                // Fallback: try to get as string
                if let Ok(val) = row.try_get::<String, _>(column_name) {
                    Some(Value::String(val))
                } else if let Ok(val) = row.try_get::<Option<String>, _>(column_name) {
                    val.map(Value::String)
                } else {
                    None
                }
            }
        }
    }

    fn build_filter_expr_fallback(
        &self,
        col1: &(String, String),
        filter: &FilterType,
        _idx: usize,
    ) -> String {
        format!("{}.{} {} ?", col1.0, col1.1, filter.to_sql())
    }

    fn bind_null<'q>(&self, query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q> {
        match kind {
            ColumnBindingKind::Varchar | ColumnBindingKind::Text | ColumnBindingKind::Unknown => {
                query.bind(None::<&str>)
            }
            ColumnBindingKind::TinyInt | ColumnBindingKind::TinyIntUnsigned => {
                query.bind(None::<i16>)
            }
            ColumnBindingKind::SmallInt => query.bind(None::<i16>),
            ColumnBindingKind::SmallIntUnsigned => query.bind(None::<i32>),
            ColumnBindingKind::Integer => query.bind(None::<i32>),
            ColumnBindingKind::IntegerUnsigned => query.bind(None::<i64>),
            ColumnBindingKind::BigInt | ColumnBindingKind::BigIntUnsigned => {
                query.bind(None::<i64>)
            }
            ColumnBindingKind::Float => query.bind(None::<f32>),
            ColumnBindingKind::Double => query.bind(None::<f64>),
            ColumnBindingKind::Boolean => query.bind(None::<bool>),
        }
    }
}

/// Get the appropriate dialect for the current backend
pub fn get_dialect() -> Box<dyn SqlDialect> {
    #[cfg(feature = "mysql")]
    return Box::new(MySqlDialect);

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    return Box::new(PostgresDialect);

    #[cfg(all(not(feature = "mysql"), not(feature = "postgres"), feature = "sqlite"))]
    return Box::new(SqliteDialect);
}
