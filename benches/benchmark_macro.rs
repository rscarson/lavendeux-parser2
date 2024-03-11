#[allow(unused_macros)]
macro_rules! generate_benches {
    ($input_file:literal) => {
        use lavendeux_parser::pest::NodeExt;
        const INPUT: &'static str = include_str!($input_file);

        fn raw_pest_pass() -> pest::iterators::Pairs<'static, lavendeux_parser::Rule> {
            <lavendeux_parser::pest::LavendeuxParser as pest::Parser<lavendeux_parser::Rule>>::parse(
                lavendeux_parser::Rule::SCRIPT,
                INPUT,
            )
            .unwrap()
        }

        fn pest_pass() -> pest::iterators::Pair<'static, lavendeux_parser::Rule> {
            lavendeux_parser::pest::LavendeuxParser::parse2(INPUT, lavendeux_parser::Rule::SCRIPT)
                .unwrap()
        }

        fn compiler_pass(
            parser: &mut lavendeux_parser::Lavendeux,
            pairs: pest::iterators::Pair<'static, lavendeux_parser::Rule>,
        ) -> lavendeux_parser::pest::Node<'static> {
            lavendeux_parser::pest::LavendeuxParser::compile_ast(pairs, parser.state_mut()).unwrap()
        }

        fn evaluate_pass(
            parser: &mut lavendeux_parser::Lavendeux,
            ast: lavendeux_parser::pest::Node<'static>,
        ) -> lavendeux_parser::polyvalue::Value {
            ast.evaluate(parser.state_mut()).unwrap()
        }

        fn criterion_benchmark(c: &mut Criterion) {
            c.bench_function("Pass 1: PEST (raw)", |b| b.iter(|| raw_pest_pass()));

            let mut parser = lavendeux_parser::Lavendeux::new(Default::default());
            c.bench_function("Pass 1: PEST (wrapper)", |b| b.iter(|| pest_pass()));

            let root_pair = pest_pass();
            c.bench_function("Pass 2: Compiler", |b| {
                b.iter(|| compiler_pass(black_box(&mut parser), black_box(root_pair.clone())))
            });

            let ast = compiler_pass(&mut parser, root_pair);
            c.bench_function("Pass 3: Executor", |b| {
                b.iter(|| evaluate_pass(black_box(&mut parser), black_box(ast.clone())))
            });
        }
    };
}
