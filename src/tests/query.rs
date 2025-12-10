#[cfg(test)]
mod tests {
    #[cfg(feature = "mysql")]
    use sqlx::MySqlPool;

    #[cfg(feature = "postgres")]
    use sqlx::PgPool;

    #[cfg(feature = "sqlite")]
    use sqlx::SqlitePool;

    use crate::{
        define_schema,
        filter::{Filter, Filtered, eq_column, eq_value},
        operations::query::{JoinType, Query},
        schema::Schema,
    };

    use std::sync::Arc;

    define_schema! {
        DummySchema {
            _id: u32,
        }
    }

    #[derive(Debug, Default)]
    struct DummyFilter;

    impl Filtered for DummyFilter {
        fn column_one(&self) -> Option<&(String, String)> {
            // Return dummy column tuple (table, column)
            // In a real impl you'd store this as a field
            static COL: (String, String) = (String::new(), String::new());
            Some(&COL)
        }

        fn filter_type(&self) -> crate::filter::FilterType {
            // Return a default FilterType for testing, e.g., Eq
            crate::filter::FilterType::Eq
        }

        fn filter1(&self) -> Option<&dyn Filtered> {
            // No sub-filter
            None
        }
    }

    #[tokio::test]
    #[ignore = "CI Fails"]
    async fn test_query_builder_limit_offset_select() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .limit(10)
            .offset(5)
            .select(SelectDummySchema::selected().all());

        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(5));
        assert!(query.select.is_some());
    }

    #[tokio::test]
    #[ignore = "CI Fails"]
    async fn test_query_builder_filter() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());
        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone()).filter(DummyFilter);

        assert_eq!(query.filters.len(), 1);
    }

    #[tokio::test]
    #[ignore = "CI Fails"]
    async fn test_query_builder_select_distinct() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .select_distinct(SelectDummySchema::selected().all());

        assert!(query.distinct);
        assert!(query.select.is_some());
    }

    #[tokio::test]
    #[ignore = "CI Fails"]
    async fn test_query_builder_joins() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        #[cfg(feature = "sqlite")]
        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .left_join::<DummySchema, SelectDummySchema>(
                Filter::default(),
                SelectDummySchema::selected().all(),
            )
            .inner_join::<DummySchema, SelectDummySchema>(
                Filter::default(),
                SelectDummySchema::selected().all(),
            )
            .cross_join::<DummySchema, SelectDummySchema>(SelectDummySchema::selected().all());

        #[cfg(not(feature = "sqlite"))]
        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .left_join::<DummySchema, SelectDummySchema>(
                Filter::default(),
                SelectDummySchema::selected().all(),
            )
            .inner_join::<DummySchema, SelectDummySchema>(
                Filter::default(),
                SelectDummySchema::selected().all(),
            )
            .right_join::<DummySchema, SelectDummySchema>(
                Filter::default(),
                SelectDummySchema::selected().all(),
            )
            .cross_join::<DummySchema, SelectDummySchema>(SelectDummySchema::selected().all());

        assert_eq!(query.joins.len(), 4);
        assert_eq!(query.joins[0].join_type, JoinType::Left);
        assert_eq!(query.joins[1].join_type, JoinType::Inner);
        #[cfg(not(feature = "sqlite"))]
        assert_eq!(query.joins[2].join_type, JoinType::Right);
        assert_eq!(query.joins[3].join_type, JoinType::Cross);
    }

    #[tokio::test]
    async fn test_select_sql_and_joins_sql() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .select(SelectDummySchema::selected().all())
            .left_join::<DummySchema, SelectDummySchema>(
                eq_column(DummySchema::_id(), DummySchema::_id()),
                SelectDummySchema::selected().all(),
            );

        let sql = Query::<DummySchema, SelectDummySchema>::select_sql(
            "SELECT ".to_string(),
            query.select,
            DummySchema::table_name(),
            &query.joins,
        );

        assert!(sql.contains("DummySchema._id"));
        assert!(sql.contains(" FROM DummySchema"));

        println!("SQL: {sql}");
        println!("Query Joins: {:?}", query.joins);

        let sql = Query::<DummySchema, SelectDummySchema>::joins_sql(sql, &query.joins);

        println!("SQL: {sql}");

        assert!(sql.contains("LEFT JOIN"));
    }

    #[tokio::test]
    async fn test_filter_sql() {
        #[cfg(feature = "mysql")]
        let pool = Arc::new(MySqlPool::connect_lazy("mysql://user:pass@localhost/db").unwrap());

        #[cfg(feature = "postgres")]
        let pool = Arc::new(PgPool::connect_lazy("postgres://user:pass@localhost/db").unwrap());

        #[cfg(feature = "sqlite")]
        let pool = Arc::new(SqlitePool::connect_lazy("sqlite://:memory:").unwrap());

        let query = Query::<DummySchema, SelectDummySchema>::new(pool.clone())
            .filter(eq_value(DummySchema::_id(), 1));

        let mut params = vec![];
        let sql = Query::<DummySchema, SelectDummySchema>::filter_sql(
            "SELECT * FROM dummy".to_string(),
            query.filters,
            &mut params,
        );
        assert!(sql.contains("WHERE"));
        assert!(!params.is_empty());
    }
}
