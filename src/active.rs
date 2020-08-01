use std::process::Command;

pub fn get_children(parent: usize) -> Vec<String> {
    let output = Command::new("pgrep")
        .arg("--list-full")
        .arg("--parent")
        .arg(parent.to_string())
        .output()
        .expect("Failed to execute pgrep");

    let mut crates = Vec::<String>::new();
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
                crates.push(crate_name.to_owned());
            }
        }
    } else {
        println!("Failed to pgrep ")
    }

    crates
}

pub fn get_active() -> Vec<String> {
    let output = Command::new("pgrep")
        .arg("cargo")
        .output()
        .expect("Failed to execute pgrep");

    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);

        if let Ok(pid) = output_str.trim().parse::<usize>() {
            return get_children(pid);
        }
    }
    Vec::<_>::new()
}
