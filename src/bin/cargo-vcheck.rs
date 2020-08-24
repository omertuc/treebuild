extern crate treebuild;
use treebuild::launch;
fn main() {
    launch(vec!["check", "--message-format=json"], "    Checking ");
}
