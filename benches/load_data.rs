use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lavendeux_parser::{
    pest::{LavendeuxParser, Node, NodeExt},
    Lavendeux, Rule,
};
use polyvalue::Value;

const INPUT: &'static str = include_str!("../example_scripts/zarban_storydata.lav");

fn pest_pass() -> pest::iterators::Pair<'static, Rule> {
    LavendeuxParser::parse2(INPUT, Rule::SCRIPT).unwrap()
}

fn compiler_pass(
    parser: &mut Lavendeux,
    pairs: pest::iterators::Pair<'static, Rule>,
) -> Node<'static> {
    LavendeuxParser::compile_ast(pairs, parser.state_mut()).unwrap()
}

fn evaluate_pass(parser: &mut Lavendeux, ast: Node<'static>) -> Value {
    ast.evaluate(parser.state_mut()).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut parser = Lavendeux::new(Default::default());
    c.bench_function("Pass 1: PEST", |b| b.iter(|| pest_pass()));

    let root_pair = pest_pass();
    c.bench_function("Pass 2: Compiler", |b| {
        b.iter(|| compiler_pass(black_box(&mut parser), black_box(root_pair.clone())))
    });

    let ast = compiler_pass(&mut parser, root_pair);
    c.bench_function("Pass 3: Executor", |b| {
        b.iter(|| evaluate_pass(black_box(&mut parser), black_box(ast.clone())))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
