use std::process::Command;

fn main() {
    println!("running");
    let status = Command::new("bash")
        .args(["validate_sfx.sh", "sfx"])
        .status()
        .expect("failed to run validate_sfx.sh");

    if !status.success() {
        panic!("WAV validation failed — see output above");
    }
}
