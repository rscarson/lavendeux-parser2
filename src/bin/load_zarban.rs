use lavendeux_parser::Lavendeux;

fn main() {
    let mut parser = Lavendeux::new(Default::default());
    let t = std::time::Instant::now();
    parser.parse("include('examples/zarbandata.lav')").unwrap();
    println!("Loaded in {}ms", t.elapsed().as_millis());
}
