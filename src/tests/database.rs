#[cfg(test)]
mod tests {
    use crate::schema::Schema;

    use crate::{database::Database, define_schema};

    define_schema! {
        Users {
            id: u64 [primary_key().not_null().auto_increment()],
            username: String [not_null().indexed()],
        }

        Posts {
            id: u64 [primary_key().not_null().auto_increment()],
            title: String [not_null().indexed()],
        }
    }

    #[tokio::test]
    async fn test_database() {
        let db = Database::connect("mysql://root:121212@localhost/noice").await;

        match db {
            Ok(db) => {
                println!("Connected to database");
            }
            Err(e) => {
                panic!("Failed to connect to database: {}", e);
            }
        }
    }

    #[tokio::test]
    #[test_retry::retry(5)]
    async fn test_list_tables_and_table_info() {
        Users::ensure_registered();
        Posts::ensure_registered();

        let tables = Database::list_tables();

        assert_eq!(tables.len(), 2); // Two of them are in other file "TestUser" and "TestDefaults"

        assert!(tables.contains(&"Users".to_string()));
        assert!(tables.contains(&"Posts".to_string()));

        let info = Database::get_table_info("Users").unwrap();
        assert_eq!(info.len(), 2);
        assert_eq!(info[0].name, "id");
        assert_eq!(info[1].name, "username");

        let info = Database::get_table_info("Posts").unwrap();
        assert_eq!(info.len(), 2);
        assert_eq!(info[0].name, "id");
        assert_eq!(info[1].name, "title");
    }

    #[test]
    fn test_generate_migration_sql() {
        Users::ensure_registered();
        Posts::ensure_registered();

        let sql = Database::generate_migration_sql();

        assert!(sql.contains("CREATE TABLE IF NOT EXISTS Users ("));
        assert!(sql.contains("CREATE TABLE IF NOT EXISTS Posts ("));

        assert!(sql.contains("id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT"));
        assert!(sql.contains("username VARCHAR(255) NOT NULL"));
        assert!(sql.contains("title VARCHAR(255) NOT NULL"));

        assert!(sql.contains("CREATE INDEX idx_Users_username ON Users (username);"));
        assert!(sql.contains("CREATE INDEX idx_Posts_title ON Posts (title);"));
    }
}
