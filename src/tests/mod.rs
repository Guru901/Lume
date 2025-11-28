pub mod database;
pub mod query;

#[cfg(test)]
#[allow(dead_code)]
mod tests {
    use crate::define_schema;
    use crate::helpers::get_starting_sql;
    use crate::row::Row;
    use crate::schema::{ColumnInfo, Schema};
    use crate::table::TableDefinition;

    // Test schema definition
    define_schema! {
        TestUser {
            id: i32 [primary_key().not_null()],
            username: String [not_null()],
            email: String,
            age: i32,
            is_active: bool [not_null()],
        }
    }

    #[test]
    fn test_schema_definition() {
        TestUser::ensure_registered();

        // Test table name
        assert_eq!(TestUser::table_name(), "TestUser");

        // Test column retrieval
        let id_col = TestUser::id();
        let username_col = TestUser::username();
        let email_col = TestUser::email();
        let age_col = TestUser::age();
        let is_active_col = TestUser::is_active();

        assert_eq!(id_col.name(), "id");
        assert_eq!(username_col.name(), "username");
        assert_eq!(email_col.name(), "email");
        assert_eq!(age_col.name(), "age");
        assert_eq!(is_active_col.name(), "is_active");

        // Test column properties
        assert!(id_col.is_primary_key());
        assert!(!id_col.is_nullable());
        assert!(!username_col.is_nullable()); // username has not_null()
        assert!(!username_col.is_primary_key());
        assert!(!is_active_col.is_nullable()); // is_active has not_null()
    }

    #[test]
    fn test_schema_columns_info() {
        TestUser::ensure_registered();

        let columns = TestUser::get_all_columns();
        assert_eq!(columns.len(), 5);

        // Check column info
        let id_info = columns.iter().find(|c| c.name == "id").unwrap();
        assert!(id_info.primary_key);
        assert!(!id_info.nullable);
        assert_eq!(id_info.data_type, "INTEGER");

        let username_info = columns.iter().find(|c| c.name == "username").unwrap();
        assert_eq!(username_info.data_type, "VARCHAR(255)");
        assert!(!username_info.primary_key);
        assert!(!username_info.nullable); // username has not_null()

        let is_active_info = columns.iter().find(|c| c.name == "is_active").unwrap();
        assert_eq!(is_active_info.data_type, "BOOLEAN");
        assert!(!is_active_info.primary_key);
        assert!(!is_active_info.nullable); // is_active has not_null()
    }

    #[test]
    fn test_row_creation_and_manipulation() {
        let mut row = Row::<TestUser>::_new();

        // Test inserting values
        let id_col = TestUser::id();
        let username_col = TestUser::username();
        let email_col = TestUser::email();
        let age_col = TestUser::age();
        let is_active_col = TestUser::is_active();

        row._insert(
            ColumnInfo {
                name: "id",
                data_type: "INTEGER",
                nullable: false,
                unique: false,
                primary_key: true,
                indexed: false,
                has_default: false,
                default_sql: None,
                auto_increment: false,
                on_update_current_timestamp: false,
                invisible: false,
                check: None,
                generated: None,
                comment: None,
                charset: None,
                collate: None,
            },
            42,
        );

        row._insert(
            ColumnInfo {
                name: "username",
                data_type: "VARCHAR(255)",
                nullable: false,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
                auto_increment: false,
                on_update_current_timestamp: false,
                invisible: false,
                check: None,
                generated: None,
                comment: None,
                charset: None,
                collate: None,
            },
            "testuser".to_string(),
        );

        row._insert(
            ColumnInfo {
                name: "email",
                data_type: "VARCHAR(255)",
                nullable: true,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
                auto_increment: false,
                on_update_current_timestamp: false,
                invisible: false,
                check: None,
                generated: None,
                comment: None,
                charset: None,
                collate: None,
            },
            "test@example.com".to_string(),
        );

        row._insert(
            ColumnInfo {
                name: "age",
                data_type: "INTEGER",
                nullable: true,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
                auto_increment: false,
                on_update_current_timestamp: false,
                invisible: false,
                check: None,
                generated: None,
                comment: None,
                charset: None,
                collate: None,
            },
            25,
        );

        row._insert(
            ColumnInfo {
                name: "is_active",
                data_type: "BOOLEAN",
                nullable: false,
                unique: false,
                primary_key: false,
                indexed: false,
                has_default: false,
                default_sql: None,
                auto_increment: false,
                on_update_current_timestamp: false,
                invisible: false,
                check: None,
                generated: None,
                comment: None,
                charset: None,
                collate: None,
            },
            true,
        );

        // Test retrieving values
        assert_eq!(row.get(id_col), Some(42));
        assert_eq!(row.get(username_col), Some("testuser".to_string()));
        assert_eq!(row.get(email_col), Some("test@example.com".to_string()));
        assert_eq!(row.get(age_col), Some(25));
        assert_eq!(row.get(is_active_col), Some(true));
    }

    #[test]
    fn test_value_conversions() {
        // Test From implementations
        let string_val: Value = "hello".to_string().into();
        let string_val_clone = string_val.clone();
        match string_val {
            Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String variant"),
        }

        let int_val: Value = 42i32.into();
        let int_val_clone = int_val.clone();
        match int_val {
            Value::Int32(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int32 variant"),
        }

        let long_val: Value = 1234567890123456789i64.into();
        let long_val_clone = long_val.clone();
        match long_val {
            Value::Int64(l) => assert_eq!(l, 1234567890123456789i64),
            _ => panic!("Expected Int64 variant"),
        }

        let float_val: Value = 3.14f64.into();
        match float_val {
            Value::Float64(f) => assert_eq!(f, 3.14),
            _ => panic!("Expected Float64 variant"),
        }

        let bool_val: Value = true.into();
        let bool_val_clone = bool_val.clone();
        match bool_val {
            Value::Bool(b) => assert_eq!(b, true),
            _ => panic!("Expected Bool variant"),
        }

        // Test TryFrom implementations
        let string_from_val: Result<String, ()> = string_val_clone.try_into();
        assert_eq!(string_from_val, Ok("hello".to_string()));

        let int_from_val: Result<i32, ()> = int_val_clone.clone().try_into();
        assert_eq!(int_from_val, Ok(42));

        let long_from_val: Result<i64, ()> = long_val_clone.try_into();
        assert_eq!(long_from_val, Ok(1234567890123456789i64));

        let bool_from_val: Result<bool, ()> = bool_val_clone.try_into();
        assert_eq!(bool_from_val, Ok(true));

        // Test cross-type conversion (i32 to i64)
        let long_from_int: Result<i64, ()> = int_val_clone.try_into();
        assert_eq!(long_from_int, Ok(42i64));
    }

    #[test]
    fn test_column_defaults() {
        define_schema! {
            TestDefaults {
                id: i32 [primary_key().not_null()],
                name: String [not_null()],
                score: i32 [default_value(100)],
                active: bool [default_value(true)],
            }
        }

        TestDefaults::ensure_registered();

        let score_col = TestDefaults::score();
        let active_col = TestDefaults::active();

        assert_eq!(score_col.get_default(), Some(&100));
        assert_eq!(active_col.get_default(), Some(&true));

        let columns = TestDefaults::get_all_columns();
        let score_info = columns.iter().find(|c| c.name == "score").unwrap();
        let active_info = columns.iter().find(|c| c.name == "active").unwrap();

        assert!(score_info.has_default);
        assert!(active_info.has_default);
        assert_eq!(score_info.default_sql, Some("100".to_string()));
        assert_eq!(active_info.default_sql, Some("TRUE".to_string()));
    }

    #[test]
    fn test_table_registry_idempotency() {
        // Test that registering the same table multiple times doesn't create duplicates
        TestUser::ensure_registered();
        TestUser::ensure_registered(); // Call again

        let tables = crate::table::get_all_tables();
        let user_tables: Vec<_> = tables
            .iter()
            .filter(|t| t.table_name() == "TestUser")
            .collect();

        assert_eq!(user_tables.len(), 1);
    }

    #[test]
    fn test_type_to_sql_string() {
        use crate::schema::type_to_sql_string;

        assert_eq!(type_to_sql_string::<String>(), "VARCHAR(255)");
        assert_eq!(type_to_sql_string::<i32>(), "INTEGER");
        assert_eq!(type_to_sql_string::<i64>(), "BIGINT");
        assert_eq!(type_to_sql_string::<f32>(), "FLOAT");
        assert_eq!(type_to_sql_string::<f64>(), "DOUBLE");
        assert_eq!(type_to_sql_string::<bool>(), "BOOLEAN");
    }

    #[test]
    fn test_schema_wrapper() {
        TestUser::ensure_registered();

        let wrapper = crate::schema::SchemaWrapper::<TestUser>::new();
        assert_eq!(wrapper.table_name(), "TestUser");

        let columns = wrapper.get_columns();
        assert_eq!(columns.len(), 5);

        let create_sql = wrapper.to_create_sql();
        assert!(create_sql.contains("CREATE TABLE IF NOT EXISTS TestUser"));
        assert!(create_sql.contains("id INTEGER PRIMARY KEY"));
        assert!(create_sql.contains("username VARCHAR(255) NOT NULL"));
        assert!(create_sql.contains("email VARCHAR(255)"));
        assert!(create_sql.contains("age INTEGER"));
        assert!(create_sql.contains("is_active BOOLEAN NOT NULL"));
    }

    #[cfg(feature = "postgres")]
    #[test]
    fn test_starting_sql_postgres() {
        use crate::StartingSql;
        assert_eq!(
            get_starting_sql(StartingSql::Select, "TestUser"),
            "SELECT ".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Insert, "TestUser"),
            "INSERT INTO TestUser (".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Delete, "TestUser"),
            "DELETE FROM TestUser ".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Update, "TestUser"),
            "UPDATE TestUser SET ".to_string()
        );
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn test_starting_sql_mysql() {
        use crate::helpers::StartingSql;

        assert_eq!(
            get_starting_sql(StartingSql::Select, "TestUser"),
            "SELECT ".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Insert, "TestUser"),
            "INSERT INTO `TestUser` (".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Delete, "TestUser"),
            "DELETE FROM `TestUser` ".to_string()
        );
        assert_eq!(
            get_starting_sql(StartingSql::Update, "TestUser"),
            "UPDATE `TestUser` SET ".to_string()
        );
    }
}

#[cfg(test)]
mod build_filter_expr_tests {
    use crate::filter::{FilterType, Filtered};
    use crate::helpers::build_filter_expr;
    use crate::schema::Value;
    use std::sync::Arc;

    #[derive(Debug)]
    struct DummyFilter {
        // For composite filters
        or: bool,
        and: bool,
        not: Option<bool>,
        filter1: Option<Arc<dyn Filtered>>,
        filter2: Option<Arc<dyn Filtered>>,
        // For simple filters
        col1: Option<(String, String)>,
        col2: Option<(String, String)>,
        filter_type: FilterType,
        value: Option<Value>,
        in_array: Option<bool>,
        array_values: Option<Vec<Value>>,
    }

    impl DummyFilter {
        fn new() -> Self {
            Self {
                or: false,
                and: false,
                not: None,
                filter1: None,
                filter2: None,
                col1: None,
                col2: None,
                filter_type: FilterType::Eq,
                value: None,
                in_array: None,
                array_values: None,
            }
        }
    }

    impl Filtered for DummyFilter {
        fn is_or_filter(&self) -> bool {
            self.or
        }
        fn is_and_filter(&self) -> bool {
            self.and
        }
        fn is_not(&self) -> Option<bool> {
            self.not
        }
        fn filter1(&self) -> Option<&dyn Filtered> {
            self.filter1.as_ref().map(|f| f.as_ref())
        }
        fn filter2(&self) -> Option<&dyn Filtered> {
            self.filter2.as_ref().map(|f| f.as_ref())
        }
        fn column_one(&self) -> Option<&(String, String)> {
            self.col1.as_ref()
        }
        fn column_two(&self) -> Option<&(String, String)> {
            self.col2.as_ref()
        }
        fn filter_type(&self) -> FilterType {
            self.filter_type
        }
        fn value(&self) -> Option<&Value> {
            self.value.as_ref()
        }
        fn is_in_array(&self) -> Option<bool> {
            self.in_array
        }
        fn array_values(&self) -> Option<&[Value]> {
            self.array_values.as_ref().map(|v| v.as_slice())
        }
    }

    #[test]
    fn test_and_or_composite_filters() {
        // (a = 1) AND (b = 2)
        let left = Arc::new(DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Eq,
            value: Some(Value::Int32(1)),
            ..DummyFilter::new()
        });
        let right = Arc::new(DummyFilter {
            col1: Some(("t".to_owned(), "b".to_owned())),
            filter_type: FilterType::Eq,
            value: Some(Value::Int32(2)),
            ..DummyFilter::new()
        });
        let and_filter = DummyFilter {
            and: true,
            filter1: Some(left.clone()),
            filter2: Some(right.clone()),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&and_filter, &mut params);
        assert_eq!(sql, "(t.a = ? AND t.b = ?)");
        assert_eq!(params, vec![Value::Int32(1), Value::Int32(2)]);

        // (a = 1) OR (b = 2)
        let or_filter = DummyFilter {
            or: true,
            filter1: Some(left),
            filter2: Some(right),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&or_filter, &mut params);
        assert_eq!(sql, "(t.a = ? OR t.b = ?)");
        assert_eq!(params, vec![Value::Int32(1), Value::Int32(2)]);
    }

    #[test]
    fn test_not_filter() {
        // NOT (a = 1)
        let inner = Arc::new(DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Eq,
            value: Some(Value::Int32(1)),
            ..DummyFilter::new()
        });
        let not_filter = DummyFilter {
            not: Some(true),
            filter1: Some(inner),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&not_filter, &mut params);
        assert_eq!(sql, "NOT (t.a = ?)");
        assert_eq!(params, vec![Value::Int32(1)]);
    }

    #[test]
    fn test_missing_filter1_filter2() {
        // Composite filter missing filter1/filter2
        let and_filter = DummyFilter {
            and: true,
            filter1: None,
            filter2: None,
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&and_filter, &mut params);
        assert_eq!(sql, "1=1");

        let not_filter = DummyFilter {
            not: Some(true),
            filter1: None,
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&not_filter, &mut params);
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn test_missing_column_one() {
        let filter = DummyFilter {
            col1: None,
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn test_in_and_not_in_array() {
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            in_array: Some(true),
            array_values: Some(vec![Value::Int32(1), Value::Int32(2)]),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a IN (?, ?)");
        assert_eq!(params, vec![Value::Int32(1), Value::Int32(2)]);

        // Empty IN array
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            in_array: Some(true),
            array_values: Some(vec![]),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "1=0");

        // NOT IN with values
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            in_array: Some(false),
            array_values: Some(vec![Value::Int32(3)]),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a NOT IN (?)");
        assert_eq!(params, vec![Value::Int32(3)]);

        // Empty NOT IN array
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            in_array: Some(false),
            array_values: Some(vec![]),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "1=1");
    }

    #[test]
    fn test_null_comparisons() {
        // IS NULL
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Eq,
            value: Some(Value::Null),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a IS NULL");
        assert!(params.is_empty());

        // IS NOT NULL
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Neq,
            value: Some(Value::Null),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a IS NOT NULL");
        assert!(params.is_empty());

        // NULL with unsupported op
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Gt,
            value: Some(Value::Null),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "1=0");
    }

    #[test]
    fn test_between() {
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Between,
            value: Some(Value::Between(
                Box::new(Value::Int32(1)),
                Box::new(Value::Int32(5)),
            )),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a BETWEEN ? AND ?");
        assert_eq!(params, vec![Value::Int32(1), Value::Int32(5)]);
    }

    #[test]
    fn test_simple_comparison() {
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            filter_type: FilterType::Gt,
            value: Some(Value::Int32(10)),
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a > ?");
        assert_eq!(params, vec![Value::Int32(10)]);
    }

    #[test]
    fn test_column_to_column_comparison() {
        let filter = DummyFilter {
            col1: Some(("t".to_owned(), "a".to_owned())),
            col2: Some(("t".to_owned(), "b".to_owned())),
            filter_type: FilterType::Eq,
            ..DummyFilter::new()
        };
        let mut params = vec![];
        let sql = build_filter_expr(&filter, &mut params);
        assert_eq!(sql, "t.a = t.b");
        assert!(params.is_empty());
    }
}
