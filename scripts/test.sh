#!/bin/bash
set -e

cargo test --all  # Run Rust tests
cargo test --no-default-features --features postgres
cargo test --no-default-features --features sqlite 