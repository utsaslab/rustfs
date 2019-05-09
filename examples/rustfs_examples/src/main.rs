extern crate rustfs2;
extern crate env_logger;

use rustfs2::FS;
use std::env;
use env_logger::Builder;

fn main() {
    Builder::new()
        .parse(&env::var("RUSTFS_EXAMPLES_LOG").unwrap_or_default())
        .init();
    
    let args: Vec<String> = env::args().collect();
    let action = &args[1];
    if action == "mount" {
        FS::new().unwrap();
    } else if action == "shutdown" {
        FS::shutdown();
    }
}
