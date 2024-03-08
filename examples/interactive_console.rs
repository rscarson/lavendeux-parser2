use lavendeux_parser::{Lavendeux, ParserOptions};
use std::collections::VecDeque;
use std::env;
use std::io::{stdin, stdout, Write};
use std::time::Duration;

/// Get the next command from the user
fn next_command() -> String {
    let mut input = String::new();
    print!("> ");
    let _ = stdout().flush();

    loop {
        stdin()
            .read_line(&mut input)
            .expect("error: unable to read user input");
        if !input.trim().ends_with('\\') || input.trim().ends_with("\\\\") {
            break;
        }
    }

    return input.trim().to_string();
}

fn main() {
    let mut lavendeux = Lavendeux::new(ParserOptions {
        timeout: Duration::from_secs(30),
        pest_call_limit: 25000000,
        ..Default::default()
    });

    // Load example scripts
    lavendeux
        .parse("include('example_scripts/zarbans_grotto.lav')")
        .expect("Could not load example scripts");

    // Preload command stack from arguments
    let mut stack: VecDeque<String> = env::args().skip(1).collect();
    if stack.is_empty() {
        println!("Ready! Type expressions below!");
    } else {
        stack.push_back("exit".to_string());
    }

    loop {
        // Make sure we have a command ready
        if stack.is_empty() {
            stack.push_back(next_command());
        }
        let cmd = stack.pop_front().unwrap();

        if cmd.is_empty() {
            continue;
        } else if ["exit", "quit"].contains(&cmd.as_str()) {
            break;
        } else {
            // Process the commands
            let t = std::time::Instant::now();
            match lavendeux.parse(&cmd) {
                Ok(values) => {
                    println!("Parsed in {}ms", t.elapsed().as_millis());
                    for value in values {
                        println!("{}", value);
                    }
                }
                Err(e) => println!("Error:\n{}", e),
            }
        }
    }
}
