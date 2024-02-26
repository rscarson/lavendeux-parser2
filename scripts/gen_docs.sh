#!/bin/bash
# This script is used to generate documentation
# It should generate documentation.md, documentation.html, and tarpaulin.html
# Usage: ./gen_docs.sh

cargo run --example interactive_console -- generate_documentation() > documentation.md
cargo tarpaulin -ohtml