use crate::{
    dialects::SqlDialect,
    filter::FilterType,
    helpers::{ColumnBindingKind, SqlBindQuery},
};

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

    fn insert_sql(&self, mut sql: String, columns: &Vec<crate::schema::ColumnInfo>) -> String {
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&self.quote_identifier(&col.name));
        }
        sql.push_str(") VALUES (");

        // Use ? for MySQL and SQLite
        for (i, _col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str("?");
        }

        sql.push_str(")");

        sql
    }
}
