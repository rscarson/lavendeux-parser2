use criterion::{black_box, criterion_group, criterion_main, Criterion};

#[macro_use]
mod benchmark_macro;

generate_benches!("../example_scripts/zarban_storydata.lav");

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
