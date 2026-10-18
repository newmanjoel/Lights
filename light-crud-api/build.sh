#!/usr/bin/env bash

set -euo pipefail

# Build the Rust project for the target platform
cargo build --release --target aarch64-unknown-linux-gnu

# Echo a success message
printf "Build succeeded: %s\n" "target/aarch64-unknown-linux-gnu/release/light-crud-api"

