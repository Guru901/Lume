#![warn(missing_docs)]

//! # Operations Module
//!
//! This module provides database operation implementations including queries and inserts.
//! It contains the core functionality for executing type-safe database operations.

/// Insert operations for adding data to database tables
pub mod insert;

/// Query operations for retrieving data from database tables
pub mod query;
