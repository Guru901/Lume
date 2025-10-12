# Changelog

## [0.7.2] - 2025-10-12

- Removed a debug print statement from the SQL query

## [0.7.1] - 2025-10-12

- Internal refactor
- Added Send and Sync traits to Filtered trait

## [0.7.0] - 2025-10-02

- Added update operation
- Added is_null, is_not_null, like, not, between, limit, ilike and offset filters
- Renamed QueryTable to SelectTable

## [0.6.0] - 2025-10-01

- Api for selecting made easier `Users::selected().id()` and `Users::selected().all()`
- Insert and Insert_many returns Result<Option<Vec<Row<T>>>, DatabaseError> now
- Added delete function
- Added in array and not in array filters
- Renamed ne to ne_value and added ne_column filter

## [0.5.0] - 2025-09-30

### Added

- **Composite Filter Support**: Introduced `OrFilter` and `AndFilter` structs, allowing filters to be combined using logical OR and AND operations.
- **Filter Combinators**: Added public helper functions for creating composite filters:
  - `or(filter1, filter2) -> OrFilter`: Combines two filters with OR logic.
  - `and(filter1, filter2) -> AndFilter`: Combines two filters with AND logic.
- **MySQL-Specific Column Features**:
  - Added `GeneratedColumn` enum with `Virtual` and `Stored` variants for MySQL generated columns.
  - Added new column methods to `Column<T>`: `auto_increment()`, `comment(text)`, `charset(charset)`, `collate(collation)`, `on_update_current_timestamp()`, `invisible()`, `check(expression)`, `generated_virtual(expression)`, `generated_stored(expression)`.
  - Added corresponding getter methods for all new attributes.

### Changed

- **SQL Generation**:
  - Enhanced `CREATE TABLE` statements to emit MySQL-specific syntax, including `AUTO_INCREMENT`, `ON UPDATE CURRENT_TIMESTAMP`, `COMMENT`, `CHARACTER SET`, `COLLATE`, `INVISIBLE`, `CHECK`, and `GENERATED` (VIRTUAL/STORED) expressions.
  - Refactored SQL filter generation to use parameterized queries for improved security and value binding.
  - Improved handling of NULL comparisons: `Eq` now generates `IS NULL`, and `Neq` generates `IS NOT NULL` when appropriate.

### Improved

- More robust support for complex filter compositions, including nested AND/OR operations.
- Enhanced SQL injection prevention through consistent use of parameterized queries.
- More reliable parameter binding and value collection in query generation.
- All new column methods use a fluent API pattern for easy chaining.
- Schema generation now fully respects MySQL-specific syntax and constraints.
- Tests updated to cover new column attributes, filter logic, and behaviors.

## [0.4.2] - 2025-09-29

- Added support for selecting columns in joins. Check the docs for more info.

## [0.4.1] - 2025-09-29

- Added support for all join types

## [0.4.0] - 2025-09-29

- Added left join support

## [0.3.0] - 2025-09-27

- Added `db.sql()` method for executing raw SQL queries
- Added `db.query().select()` method for specifying which columns to select
- `define_schema!` macro now supports multiple schemas with just one marco call

## [0.2.2] - 2025-09-27

- Fixed wrong readme examples

## [0.2.1] - 2025-09-27

- Fixed wrong readme examples

## [0.2.0] - 2025-09-27

- Added mutations support.
- Added `db.mutation()` method for executing mutations.
- Added docs

## [0.1.2] - 2025-09-24

- Added details in `Cargo.toml`.
- Internal refactoring.

## [0.1.1] - 2025-09-24 (Initial Release)

### Added

- Basic schema definition macro `define_schema!`
- Type-safe column definitions with metadata
- MySQL database connection support
- Basic query builder with filtering capabilities
- Row abstraction with type-safe value retrieval
- Table registry system for schema management
- Support for common SQL types: `String`, `i32`, `i64`, `f64`, `bool`
- Column constraints: primary key, not null, unique, indexed, default values
- SQL generation for CREATE TABLE statements
- Basic filtering system with comparison operators

### Features

- Ergonomic schema definition inspired by Drizzle ORM
- Type-safe database operations at compile time
- Automatic SQL generation from Rust types
- Extensible column constraint system
- MySQL-specific optimizations

### Dependencies

- `sqlx` 0.8.6 for MySQL database connectivity
- `tokio` 1.47.1 for async runtime support

## [0.1.0] - Name registered
