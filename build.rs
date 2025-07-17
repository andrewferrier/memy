use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["describe", "--tags", "--always", "--dirty"])
        .output()
        .expect("Failed to execute git");

    let git_version = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_VERSION={}", git_version.trim());

    // Invalidate the build if HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs/");
}
