use lavendeux_parser::{Lavendeux, ParserOptions};

fn main() {
    let mut lavendeux = Lavendeux::new(ParserOptions {
        pest_call_limit: 25000000,
        ..Default::default()
    });
    match lavendeux.parse("include('examples/stdlib_example.lav')") {
        Ok(_) => {
            println!("Successfully loaded stdlib - no errors!");
        }
        Err(e) => {
            panic!("Failed to load stdlib: {}", e);
        }
    }
}
