# Joins

Lume provides left, inner, and right joins with typed column access.

```rust
use lume::filter::eq_column;

let rows = db
    .query::<Author, SelectAuthor>()
    .select(SelectAuthor::selected().name())
    .left_join::<Book, QueryBook>(
        eq_column(Author::id(), Book::author_id()),
        QueryBook { title: true, ..Default::default() },
    )
    .execute()
    .await?;
```

- `left_join::<T, SelectT>(on, selection)`
- `inner_join::<T, SelectT>(on, selection)`
- `right_join::<T, SelectT>(on, selection)`
