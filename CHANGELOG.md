# Changelog

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
