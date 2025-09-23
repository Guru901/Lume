use crate::{
    operations::query::Query,
    schema::{ColumnInfo, Schema},
    table::get_all_tables,
};

pub struct Database;

impl Database {
    pub fn query<T: Schema>(&self) -> Query<T> {
        Query::new()
    }

    pub fn register_table<T: Schema>(&self) {
        T::ensure_registered();
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
}

#[derive(Debug)]
pub struct DatabaseError;

pub async fn connect(_url: &str) -> Result<Database, DatabaseError> {
    Ok(Database {})
}
