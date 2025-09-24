use std::{fmt::Debug, sync::Arc};

use crate::{
    operations::query::Query,
    schema::{ColumnInfo, Schema},
    table::get_all_tables,
};
use sqlx::MySqlPool;

pub struct Database {
    connection: Arc<MySqlPool>,
}

impl Database {
    pub fn query<T: Schema + Debug>(&self) -> Query<T> {
        Query::new(Arc::clone(&self.connection))
    }

    pub async fn register_table<T: Schema>(&self) -> Result<(), DatabaseError> {
        T::ensure_registered();
        let sql = Database::generate_migration_sql();
        for stmt in sql.split(';').map(str::trim).filter(|s| !s.is_empty()) {
            sqlx::query(stmt)
                .execute(&*self.connection)
                .await
                .map_err(DatabaseError)?;
        }
        Ok(())
    }

    pub fn generate_migration_sql() -> String {
        let tables = get_all_tables();
        tables
            .iter()
            .map(|table| table.to_create_sql())
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    // Get schema information
    pub fn get_table_info(table_name: &str) -> Option<Vec<ColumnInfo>> {
        let tables = get_all_tables();
        tables
            .iter()
            .find(|table| table.table_name() == table_name)
            .map(|table| table.get_columns())
    }

    // List all registered tables
    pub fn list_tables() -> Vec<String> {
        let tables = get_all_tables();
        tables
            .iter()
            .map(|table| table.table_name().to_string())
            .collect()
    }

    pub async fn connect(url: &str) -> Result<Database, DatabaseError> {
        let conn = MySqlPool::connect(url).await.map_err(DatabaseError)?;
        Ok(Database {
            connection: Arc::new(conn),
        })
    }
}

#[derive(Debug)]
pub struct DatabaseError(sqlx::Error);
