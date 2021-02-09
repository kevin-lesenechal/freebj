use std::process::Command;
use serde_json::Value;

pub fn run_freebj(args: &[&str]) -> Value {
    let mut bin = std::env::current_dir().unwrap();
    bin.push("target/debug/freebj");

    let proc = Command::new(bin)
        .args(args)
        .output().expect("Couldn't launch freebj");

    if !proc.status.success() {
        panic!("freebj exited with status {}", proc.status);
    }

    serde_json::from_slice(&proc.stdout).expect("Couldn't parse JSON")
}
