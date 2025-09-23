use std::sync::{Mutex, OnceLock};

use crate::schema::{ColumnInfo, Schema, SchemaWrapper};

static TABLE_REGISTRY: OnceLock<Mutex<Vec<Box<dyn TableDefinition>>>> = OnceLock::new();

pub trait TableDefinition: Send + Sync {
    fn table_name(&self) -> &'static str;
    fn get_columns(&self) -> Vec<ColumnInfo>;
    fn to_create_sql(&self) -> String;
    fn clone_box(&self) -> Box<dyn TableDefinition>;
}

// Registry functions
pub fn register_table<T: Schema + Send + Sync + 'static>() {
    let registry = TABLE_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    let mut tables = registry.lock().unwrap();

    tables.push(Box::new(SchemaWrapper::<T>::new()));
}

pub fn get_all_tables() -> Vec<Box<dyn TableDefinition>> {
    let registry = TABLE_REGISTRY.get_or_init(|| Mutex::new(Vec::new()));
    let tables = registry.lock().unwrap();
    tables.iter().map(|t| t.clone_box()).collect()
}
