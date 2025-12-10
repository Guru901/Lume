// src/backends/mod.rs

/// Trait for database-specific SQL generation
pub trait SqlDialect {
    /// Quote an identifier (table name, column name) according to backend rules
    fn quote_identifier(&self, identifier: &str) -> String;

    /// Generate a placeholder for the given index (0-based)
    fn placeholder(&self, index: usize) -> String;

    /// Adapt SQL syntax for backend-specific requirements
    fn adapt_sql(&self, sql: String) -> String;
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
}

/// Get the appropriate dialect for the current backend
pub fn get_dialect() -> Box<dyn SqlDialect> {
    #[cfg(feature = "mysql")]
    return Box::new(MySqlDialect);

    #[cfg(all(not(feature = "mysql"), feature = "postgres"))]
    return Box::new(PostgresDialect);

    #[cfg(all(not(feature = "mysql"), not(feature = "postgres"), feature = "sqlite"))]
    return Box::new(SqliteDialect);

    #[cfg(all(not(feature = "mysql"), not(feature = "postgres"), not(feature = "sqlite")))]
    compile_error!("At least one database backend feature (mysql, postgres, sqlite) must be enabled");
}
