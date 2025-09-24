mod column;

use std::marker::PhantomData;

use crate::table::TableDefinition;
pub use column::Column;
pub use column::Value;

pub trait Schema {
    fn table_name() -> &'static str;
    fn get_all_columns() -> Vec<ColumnInfo>;
    fn ensure_registered();
}

#[derive(Debug, Clone)]
pub struct ColumnInfo {
    pub name: &'static str,
    pub data_type: &'static str,
    pub nullable: bool,
    pub unique: bool,
    pub primary_key: bool,
    pub indexed: bool,
    pub has_default: bool,
    pub default_sql: Option<String>,
}

#[macro_export]
macro_rules! define_schema {
    (
        $struct_name:ident {
            $(
                $name:ident: $type:ty $([ $($args:tt)* ])?
            ),* $(,)?
        }
    ) => {
        #[derive(Debug)]
        pub struct $struct_name;

        impl $struct_name {
            $(
                pub fn $name() -> &'static Column<$type> {
                    static COL: std::sync::OnceLock<Column<$type>> = std::sync::OnceLock::new();
                    COL.get_or_init(|| {
                        Column::<$type>::new(stringify!($name), stringify!($struct_name))
                            $(.$($args)*)?
                    })
                }
            )*
        }


        // Auto-register the table when the struct is defined
        #[allow(non_upper_case_globals)]
        static _REGISTER: std::sync::Once = std::sync::Once::new();
        use lume::table::register_table;
        use lume::schema::type_to_sql_string;
        use lume::schema::DefaultToSql;

        impl Schema for $struct_name {
            fn table_name() -> &'static str {
                stringify!($struct_name)
            }

            fn ensure_registered() {
                _REGISTER.call_once(|| {
                    register_table::<$struct_name>();
                });
            }

            fn get_all_columns() -> Vec<ColumnInfo> {
                vec![
                    $(
                        {
                            let col = Self::$name();
                            ColumnInfo {
                                name: col.name(),
                                data_type: type_to_sql_string::<$type>(),
                                nullable: col.is_nullable(),
                                unique: col.is_unique(),
                                primary_key: col.is_primary_key(),
                                indexed: col.is_indexed(),
                                has_default: col.get_default().is_some(),
                                default_sql: col.default_to_sql(),
                            }
                        }
                    ),*
                ]
            }
        }
    };
}

pub fn type_to_sql_string<T: 'static>() -> &'static str {
    use std::any::TypeId;

    let type_id = TypeId::of::<T>();

    if type_id == TypeId::of::<String>() {
        "TEXT"
    } else if type_id == TypeId::of::<i32>() {
        "INTEGER"
    } else if type_id == TypeId::of::<i64>() {
        "BIGINT"
    } else if type_id == TypeId::of::<f32>() {
        "REAL"
    } else if type_id == TypeId::of::<f64>() {
        "DOUBLE PRECISION"
    } else if type_id == TypeId::of::<bool>() {
        "BOOLEAN"
    } else {
        "TEXT" // fallback
    }
}

pub(crate) struct SchemaWrapper<T: Schema> {
    _phantom: PhantomData<T>,
}

// Implement Clone for SchemaWrapper<T>
impl<T: Schema> Clone for SchemaWrapper<T> {
    fn clone(&self) -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Schema> SchemaWrapper<T> {
    pub(crate) fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<T: Schema + Sync + Send + 'static> TableDefinition for SchemaWrapper<T> {
    fn table_name(&self) -> &'static str {
        T::table_name()
    }

    fn get_columns(&self) -> Vec<ColumnInfo> {
        T::get_all_columns()
    }

    fn to_create_sql(&self) -> String {
        let table_name = self.table_name();
        let columns = self.get_columns();

        let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", table_name);

        let column_definitions: Vec<String> = columns
            .iter()
            .map(|col| {
                let mut def = format!("    {} {}", col.name, col.data_type);

                if col.primary_key {
                    def.push_str(" PRIMARY KEY");
                }

                if !col.nullable && !col.primary_key {
                    def.push_str(" NOT NULL");
                }

                if col.unique && !col.primary_key {
                    def.push_str(" UNIQUE");
                }

                if let Some(ref default) = col.default_sql {
                    def.push_str(&format!(" DEFAULT {}", default));
                }

                def
            })
            .collect();

        sql.push_str(&column_definitions.join(",\n"));
        sql.push_str("\n);");

        // Add indexes
        let indexes: Vec<String> = columns
            .iter()
            .filter(|col| col.indexed && !col.primary_key)
            .map(|col| {
                format!(
                    "CREATE INDEX idx_{}_{} ON {} ({});",
                    table_name, col.name, table_name, col.name
                )
            })
            .collect();

        if !indexes.is_empty() {
            sql.push_str("\n\n");
            sql.push_str(&indexes.join("\n"));
        }

        sql
    }

    fn clone_box(&self) -> Box<dyn TableDefinition> {
        Box::new(self.clone())
    }
}

pub trait DefaultToSql {
    fn default_to_sql(&self) -> Option<String>;
}

// Implement for each column type
impl DefaultToSql for Column<String> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default()
            .map(|v| format!("'{}'", v.replace('\'', "''")))
    }
}

impl DefaultToSql for Column<i32> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<i64> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<f32> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<f64> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| v.to_string())
    }
}

impl DefaultToSql for Column<bool> {
    fn default_to_sql(&self) -> Option<String> {
        self.get_default().map(|v| {
            if *v {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        })
    }
}
