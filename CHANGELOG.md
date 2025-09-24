# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Comprehensive unit test suite covering schema definition, row manipulation, and value conversions
- Function-local static registration to prevent macro expansion collisions
- Idempotent table registration to prevent duplicate entries
- Enhanced `Value` enum with `Long(i64)` variant for better type support
- Type-safe row extraction from MySQL rows using data type information
- Cross-type conversions (e.g., `i32` to `i64`) in value system

### Changed

- **BREAKING**: Updated `Value` enum to include `Long(i64)` variant for better 64-bit integer support
- Improved MySQL row extraction to use column data types instead of generic type inference
- Enhanced macro system to use `$crate` for better module path resolution
- Updated type-to-SQL mapping for better MySQL compatibility

### Fixed

- Fixed MySQL row extraction issue where generic type inference was failing
- Resolved macro static collision when multiple schemas are defined in the same module
- Fixed table registry allowing duplicate entries
- Improved error handling in row value extraction

### Technical Improvements

- Added proper `TryFrom` implementations for `i64` values
- Enhanced column metadata with better SQL type mapping
- Improved memory efficiency with function-local statics
- Better type safety in row operations

## [0.1.0] - Initial Release

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

## Development Notes

### Testing Strategy

- Unit tests cover core functionality including schema definition, row manipulation, and value conversions
- Tests verify type safety and proper SQL generation
- Registry idempotency tests ensure no duplicate table registrations
- Cross-type conversion tests validate value system flexibility

### Performance Considerations

- Function-local statics reduce memory overhead compared to module-level statics
- Idempotent registration prevents unnecessary allocations
- Type-safe operations eliminate runtime type checking overhead

### Future Roadmap

- PostgreSQL and SQLite database support
- Advanced query operations (JOINs, aggregations)
- Migration system for schema evolution
- Connection pooling and transaction support
- Performance benchmarking and optimization
