@cd scripts
@cd ..

cargo test
cargo test --no-default-features --lib
cargo test --all-features
cargo fmt --check
cargo clippy