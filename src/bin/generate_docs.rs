//! This is a simple binary that generates the documentation for the Lavendeux parser.
//! Regenerates the contents of `documentation.md`, and `documentation.html`
use lavendeux_parser::Lavendeux;
use std::process::Command;

fn run_command(cmd: &str) {
    let mut parts = cmd.split_whitespace().collect::<Vec<_>>();
    let cmd = parts.remove(0);

    let mut child = Command::new(cmd)
        .args(&parts)
        .spawn()
        .expect("Failed to start command");
    let ecode = child.wait().expect("Failed to wait on command");
    assert!(ecode.success());
}

fn main() {
    let parser = Lavendeux::new(Default::default());
    let docs = parser.generate_documentation();
    std::fs::write("documentation.md", docs).expect("Failed to write documentation.md");
    run_command(&format!(
        "rustdoc documentation.md --o ./ --html-before-content=src/bin/documentation_template.html"
    ))
}
