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

    pub async fn register_table<T: Schema>(&mut self) {
        T::ensure_registered();
        let mut conn = self.connection.acquire().await.unwrap();

        sqlx::query(&Database::generate_migration_sql())
            .execute(&mut *conn)
            .await
            .unwrap();
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
        match MySqlPool::connect(url).await {
            Ok(conn) => {
                return Ok(Database {
                    connection: Arc::new(conn),
                });
            }
            Err(e) => return Err(DatabaseError),
        }
    }
}

#[derive(Debug)]
pub struct DatabaseError;
