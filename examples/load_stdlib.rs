use lavendeux_parser::Lavendeux;

fn main() {
    let mut lavendeux = Lavendeux::new(Default::default());
    match lavendeux.parse("include('examples/stdlib_example.lav')") {
        Ok(_) => {
            println!("Successfully loaded stdlib - no errors!");
        }
        Err(e) => {
            panic!("Failed to load stdlib: {}", e);
        }
    }
}
