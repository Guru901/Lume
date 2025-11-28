use crate::{
    filter::Filtered,
    schema::{ColumnInfo, Value},
};

#[cfg(feature = "mysql")]
use sqlx::{MySql, mysql::MySqlArguments};
#[cfg(feature = "postgres")]
use sqlx::{Postgres, postgres::PgArguments};

#[derive(PartialEq, Debug)]
pub(crate) enum StartingSql {
    Select,
    Insert,
    Delete,
    Update,
}

pub(crate) fn get_starting_sql(starting_sql: StartingSql, table_name: &str) -> String {
    let table_ident = quote_identifier(table_name);
    match starting_sql {
        StartingSql::Select => "SELECT ".to_string(),
        StartingSql::Insert => format!("INSERT INTO {} (", table_ident),
        StartingSql::Delete => format!("DELETE FROM {} ", table_ident),
        StartingSql::Update => format!("UPDATE {} SET ", table_ident),
    }
}

pub(crate) fn quote_identifier(identifier: &str) -> String {
    #[cfg(feature = "mysql")]
    {
        return format!("`{}`", identifier.replace('`', "``"));
    }

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    {
        return format!("\"{}\"", identifier.replace('"', "\"\""));
    }
}

#[cfg(not(feature = "mysql"))]
pub(crate) fn returning_sql(mut sql: String, returning: &Vec<&'static str>) -> String {
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

#[cfg(feature = "mysql")]
pub(crate) fn returning_sql(mut sql: String, returning: &Vec<&'static str>) -> String {
    if returning.is_empty() {
        return sql;
    }
    // MySQL does not support RETURNING clause in the same way as PostgreSQL.
    // Return SQL unchanged or implement alternative strategy (e.g., SELECT LAST_INSERT_ID()).
    sql
}

pub(crate) fn build_filter_expr(filter: &dyn Filtered, params: &mut Vec<Value>) -> String {
    if filter.is_or_filter() || filter.is_and_filter() {
        let op = if filter.is_or_filter() { "OR" } else { "AND" };
        let Some(f1) = filter.filter1() else {
            eprintln!("Warning: Composite filter missing filter1, using tautology");
            return "1=1".to_string();
        };
        let Some(f2) = filter.filter2() else {
            eprintln!("Warning: Composite filter missing filter2, using tautology");
            return "1=1".to_string();
        };
        let left = build_filter_expr(f1, params);
        let right = build_filter_expr(f2, params);
        return format!("({} {} {})", left, op, right);
    }

    if filter.is_not().unwrap_or(false) {
        let Some(f) = filter.filter1() else {
            eprintln!("Warning: Not filter missing filter1, using tautology");
            return "1=1".to_string();
        };
        return format!("NOT ({})", build_filter_expr(f, params));
    }

    let Some(col1) = filter.column_one() else {
        eprintln!("Warning: Simple filter missing column_one, using tautology");
        return "1=1".to_string();
    };
    // Handle IN / NOT IN array filters
    if let Some(in_array) = filter.is_in_array() {
        let values = filter.array_values().unwrap_or(&[]);
        if values.is_empty() {
            return if in_array {
                "1=0".to_string()
            } else {
                "1=1".to_string()
            };
        }

        let start_idx = params.len();
        let mut placeholders: Vec<String> = Vec::with_capacity(values.len());
        for (i, v) in values.iter().cloned().enumerate() {
            params.push(v);
            #[cfg(feature = "mysql")]
            placeholders.push("?".to_string());
            #[cfg(feature = "postgres")]
            placeholders.push(format!("${}", start_idx + i + 1));
        }

        let op = if in_array { "IN" } else { "NOT IN" };
        return format!("{}.{} {} ({})", col1.0, col1.1, op, placeholders.join(", "));
    }
    if let Some(value) = filter.value() {
        match value {
            Value::Null => {
                // Special handling for NULL comparisons
                let op = filter.filter_type();
                let null_sql = match op {
                    crate::filter::FilterType::Eq => "IS NULL",
                    crate::filter::FilterType::Neq => "IS NOT NULL",
                    _ => {
                        // Unsupported operator with NULL; force false to avoid surprising results
                        return "1=0".to_string();
                    }
                };
                format!("{}.{} {}", col1.0, col1.1, null_sql)
            }
            Value::Between(min, max) => {
                params.push((**min).clone());
                params.push((**max).clone());
                format!("{}.{} BETWEEN ? AND ?", col1.0, col1.1)
            }
            _ => {
                params.push(value.clone());
                format!("{}.{} {} ?", col1.0, col1.1, filter.filter_type().to_sql())
            }
        }
    } else if let Some(col2) = filter.column_two() {
        format!(
            "{}.{} {} {}.{}",
            col1.0,
            col1.1,
            filter.filter_type().to_sql(),
            col2.0,
            col2.1
        )
    } else {
        return "1=1".to_string();
    }
}

#[cfg(feature = "mysql")]
pub(crate) type SqlBindQuery<'q> = sqlx::query::Query<'q, MySql, MySqlArguments>;

#[cfg(feature = "postgres")]
pub(crate) type SqlBindQuery<'q> = sqlx::query::Query<'q, Postgres, PgArguments>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum ColumnBindingKind {
    Varchar,
    Text,
    TinyInt,
    SmallInt,
    Integer,
    BigInt,
    TinyIntUnsigned,
    SmallIntUnsigned,
    IntegerUnsigned,
    BigIntUnsigned,
    Float,
    Double,
    Boolean,
    Unknown,
}

impl ColumnBindingKind {
    fn from_column(column: &ColumnInfo) -> Self {
        match column.data_type {
            "VARCHAR(255)" => ColumnBindingKind::Varchar,
            "TEXT" => ColumnBindingKind::Text,
            "TINYINT" => ColumnBindingKind::TinyInt,
            "SMALLINT" => ColumnBindingKind::SmallInt,
            "INTEGER" => ColumnBindingKind::Integer,
            "BIGINT" => ColumnBindingKind::BigInt,
            "TINYINT UNSIGNED" => ColumnBindingKind::TinyIntUnsigned,
            "SMALLINT UNSIGNED" => ColumnBindingKind::SmallIntUnsigned,
            "INTEGER UNSIGNED" => ColumnBindingKind::IntegerUnsigned,
            "BIGINT UNSIGNED" => ColumnBindingKind::BigIntUnsigned,
            "FLOAT" => ColumnBindingKind::Float,
            "DOUBLE" => ColumnBindingKind::Double,
            "BOOLEAN" => ColumnBindingKind::Boolean,
            _ => ColumnBindingKind::Unknown,
        }
    }
}

/// Binds an optional value for a specific column, falling back to NULL binding when needed.
pub(crate) fn bind_column_value<'q>(
    query: SqlBindQuery<'q>,
    column: &ColumnInfo,
    value: Option<&Value>,
) -> SqlBindQuery<'q> {
    let kind = ColumnBindingKind::from_column(column);
    match value {
        None => bind_null(query, kind),
        Some(Value::Null) => bind_null(query, kind),
        Some(Value::Array(_)) => bind_null(query, kind),
        Some(other) => bind_value(query, other.clone()),
    }
}

/// Binds a generic [`Value`] into the provided SQLx query, handling backend differences.
pub(crate) fn bind_value<'q>(query: SqlBindQuery<'q>, value: Value) -> SqlBindQuery<'q> {
    match value {
        Value::String(s) => query.bind(s),
        Value::Int8(i) => query.bind(i),
        Value::Int16(i) => query.bind(i),
        Value::Int32(i) => query.bind(i),
        Value::Int64(i) => query.bind(i),

        #[cfg(feature = "mysql")]
        Value::UInt8(u) => query.bind(u),

        #[cfg(feature = "postgres")]
        Value::UInt8(u) => query.bind(u as i16),

        #[cfg(feature = "postgres")]
        Value::UInt16(u) => query.bind(u as i32),
        #[cfg(feature = "postgres")]
        Value::UInt32(u) => query.bind(u as i64),
        #[cfg(feature = "postgres")]
        Value::UInt64(u) => {
            debug_assert!(
                u <= i64::MAX as u64,
                "UInt64 value exceeds i64::MAX, data loss will occur"
            );
            query.bind(u as i64)
        }

        #[cfg(feature = "mysql")]
        Value::UInt16(u) => query.bind(u),
        #[cfg(feature = "mysql")]
        Value::UInt32(u) => query.bind(u),
        #[cfg(feature = "mysql")]
        Value::UInt64(u) => query.bind(u),
        Value::Float32(f) => query.bind(f),
        Value::Float64(f) => query.bind(f),
        Value::Bool(b) => query.bind(b),
        Value::Between(min, max) => {
            let query = bind_value(query, *min);
            bind_value(query, *max)
        }
        Value::Array(_arr) => {
            eprintln!("Warning: Attempted to bind Value::Array, which is not supported. Skipping.");
            query
        }
        Value::Null => query,
    }
}

#[cfg(feature = "mysql")]
fn bind_null<'q>(query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q> {
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

#[cfg(feature = "postgres")]
fn bind_null<'q>(query: SqlBindQuery<'q>, kind: ColumnBindingKind) -> SqlBindQuery<'q> {
    match kind {
        ColumnBindingKind::Varchar
        | ColumnBindingKind::Text
        | ColumnBindingKind::TinyIntUnsigned
        | ColumnBindingKind::Unknown => query.bind(None::<&str>),
        ColumnBindingKind::TinyInt => query.bind(None::<i8>),
        ColumnBindingKind::SmallInt => query.bind(None::<i16>),
        ColumnBindingKind::SmallIntUnsigned => query.bind(None::<i32>),
        ColumnBindingKind::Integer => query.bind(None::<i32>),
        ColumnBindingKind::IntegerUnsigned => query.bind(None::<i64>),
        ColumnBindingKind::BigInt | ColumnBindingKind::BigIntUnsigned => query.bind(None::<i64>),
        ColumnBindingKind::Float => query.bind(None::<f32>),
        ColumnBindingKind::Double => query.bind(None::<f64>),
        ColumnBindingKind::Boolean => query.bind(None::<bool>),
    }
}
