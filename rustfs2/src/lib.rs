#![feature(uniform_paths)]
#![feature(async_await, await_macro, futures_api)]

#[macro_use]
extern crate arrayref;
extern crate spdk_rs;
extern crate time;
#[macro_use]
extern crate failure;
extern crate nix;
#[macro_use]
extern crate log;
extern crate bincode;
#[macro_use]
extern crate serde_derive;
extern crate serde;
#[macro_use]
extern crate lazy_static;

mod bitmap;
mod constants;
mod device;
mod file;
mod fs;

pub use fs::FS;
