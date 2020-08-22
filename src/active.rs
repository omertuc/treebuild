use std::{collections::HashSet, process::Command};

pub fn get_children(parent: usize) -> HashSet<String> {
    let output = Command::new("pgrep")
        .arg("--list-full")
        .arg("--parent")
        .arg(parent.to_string())
        .output()
        .expect("Failed to execute pgrep");

    let mut crates = HashSet::<String>::new();
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let mut fields_iter = line.split_whitespace();
            for field in &mut fields_iter {
                if field == "--crate-name" {
                    break;
                }
            }

            if let Some(crate_name) = fields_iter.next() {
                crates.insert(crate_name.replace("_", "-").to_owned());
            }
        }
    }
    crates
}

pub fn get_active() -> HashSet<String> {
    let output = Command::new("pgrep")
        .arg("cargo")
        .arg("--parent")
        .arg(std::process::id().to_string())
        .arg("--exact")
        .output()
        .expect("Failed to execute pgrep");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let trimmed = output_str.trim();

        if let Ok(pid) = trimmed.parse::<usize>() {
            return get_children(pid);
        }
    }

    HashSet::<_>::new()
}
