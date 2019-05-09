#![feature(uniform_paths)]
#![feature(async_await, await_macro, futures_api)]

#[macro_use]
extern crate arrayref;
extern crate spdk_rs;
extern crate time;
#[macro_use]
extern crate failure;
extern crate nix;

mod constants;
mod fs;

pub use fs::FS;
