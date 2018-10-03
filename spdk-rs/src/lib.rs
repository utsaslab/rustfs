//pub extern crate libspdk_sys as raw;
extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;

mod event;

pub use event::{AppOpts};


fn main() {
    let mut opts :raw::spdk_app_opts;
}
