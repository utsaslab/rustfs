extern crate rustfs2;

use rustfs2::FS;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    let action = &args[1];
    if action == "mount" {
        FS::new().unwrap();
    } else if action == "shutdown" {
        FS::shutdown();
    }
}
