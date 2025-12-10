use crate::{
    dialects::SqlDialect,
    filter::FilterType,
    helpers::{ColumnBindingKind, SqlBindQuery},
};

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

    fn insert_sql(&self, mut sql: String, columns: &Vec<crate::schema::ColumnInfo>) -> String {
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                sql.push_str(", ");
            }
            sql.push_str(&self.quote_identifier(&col.name));
        }
        sql.push_str(") VALUES (");

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
