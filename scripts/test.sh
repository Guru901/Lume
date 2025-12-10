#!/bin/bash
set -e

cargo test --all --no-default-features --features mysql
cargo test --all --no-default-features --features postgres
cargo test --all --no-default-features --features sqlite