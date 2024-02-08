#!/bin/bash
# This script is used to test the target program with the input file
# It is meant to mirror the github actions workflow
# Usage: ./test.sh

# If we are in the scripts directory, move to the parent directory
if [ -f "fuzzer.sh" ]; then
    cd ..
fi

cargo test --verbose
cargo test --no-default-features --lib
cargo test --all-features
cargo fmt --check
cargo clippy