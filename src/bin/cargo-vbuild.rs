extern crate treebuild;
use treebuild::launch;
fn main() {
    launch(vec!["build", "--message-format=json"], "   Compiling ");
}
