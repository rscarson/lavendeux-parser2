//! This script runs `cargo tarpaulin` and opens the generated HTML report in the default browser.
//! Used to regenerate the code-coverage report

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
    // run `cargo tarpaulin -ohtml`
    // open `tarpaulin-report.html` in your browser
    run_command("cargo tarpaulin -ohtml");
    run_command("open tarpaulin-report.html");
}
