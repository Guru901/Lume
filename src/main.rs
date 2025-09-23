use lume::database::{DatabaseError, connect};
use lume::define_columns;
use lume::schema::Column;
use lume::schema::Schema;

define_columns! {
    Users {
        username: String,
        id: i32 [primary_key().not_null()]
    }

    Posts {
        id: i32 [primary_key()],
        title: String [not_null()],
        content: String,
    }
}

#[tokio::main]
async fn main() -> Result<(), DatabaseError> {
    let db = connect("postgresql://...");

    let rows = db
        .query::<Users>()
        .filter(Users::username(), "guru".to_string())
        .await?;

    for row in rows {
        let username: Option<String> = row.get(Users::username());

        println!("User: {:?}", username);

        println!("Username is nullable: {}", Users::username().is_nullable());
    }

    Ok(())
}
