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
                crates.insert(crate_name.to_owned());
            }
        }
    } else {
        println!("Failed to pgrep on cargo children")
    }

    crates
}

pub fn get_active() -> HashSet<String> {
    let output = Command::new("pgrep")
        .arg("cargo")
        .arg("--exact")
        .output()
        .expect("Failed to execute pgrep");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        let trimmed = output_str.trim();

        if let Ok(pid) = trimmed.parse::<usize>() {
            return get_children(pid);
        } else {
            println!("Failed to get cargo pid from pgrep output {}", trimmed);
        }
    } else {
        println!("Failed to execute pgrep on cargo");
    }

    HashSet::<_>::new()
}
