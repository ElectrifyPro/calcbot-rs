use std::process::Command;

fn main() {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .unwrap();
    let commit_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=COMMIT_HASH={commit_hash}");
}

