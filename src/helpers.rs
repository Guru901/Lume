use crate::{filter::Filtered, schema::Value};

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
        return format!("`{}`", identifier);
    }

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    {
        return format!("\"{}\"", identifier);
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
    sql.push_str(&returning.join(", "));

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
        let mut placeholders: Vec<&'static str> = Vec::with_capacity(values.len());
        for v in values.iter().cloned() {
            params.push(v);
            placeholders.push("?");
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
