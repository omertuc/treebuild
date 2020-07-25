use std::io::Read;
use std::io::{self, Write};
use std::process::Command;
use std::thread;
use std::time::Duration;
use sysinfo::System;

pub fn launch() {
    // thread::spawn(|| {
    let build_dir = "/home/omer/repos/cargo";

    Command::new("cargo").arg("clean").current_dir(build_dir);

    let child = Command::new("cargo")
        .arg("run")
        .arg("--color")
        .arg("never")
        .current_dir(build_dir)
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();

    let child_stderr = child.stderr.unwrap();

    let reader = std::io::BufReader::new(child_stderr);

    for bytes in reader.bytes() {
        println!("{:?}", bytes.unwrap());
    }

    // });
}

fn get_active() -> Vec<String> {
    // let system = System::new_all();

    // // First we update all information of our system struct.
    // system.refresh_all();

    // // Now let's print every process' id and name:
    // for process in system.get_process_by_name("rustc") {
    //     println!("{} {}", process.pid(), process.name());
    // }

    Vec::<_>::new()
}
