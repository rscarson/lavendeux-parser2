use lavendeux_parser2::{Error, Lavendeux};
use std::collections::VecDeque;
use std::env;
use std::io::{stdin, stdout, Write};

/// Get the next command from the user
fn next_command() -> String {
    let mut input = String::new();
    print!("> ");
    let _ = stdout().flush();
    stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    return input.trim().to_string();
}

fn main() -> Result<(), Error> {
    let mut lavendeux = Lavendeux::new();

    // Load extensions
    lavendeux.load_extension("example_extensions/simple_extension.js")?;
    lavendeux.load_extension("example_extensions/zarbans_grotto.js")?;

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

        if cmd.len() == 0 {
            continue;
        } else if ["exit", "quit"].contains(&cmd.as_str()) {
            break;
        } else {
            // Process the commands
            match lavendeux.parse(&cmd) {
                Ok(values) => {
                    for value in values {
                        println!("{}", value);
                    }
                }
                Err(e) => println!("Error: {}", e),
            }
        }
    }

    Ok(())
}
