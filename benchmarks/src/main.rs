//! Main driver program to run various benchmarks
#![feature(await_macro, async_await, futures_api, nll, generators)]

extern crate failure;
extern crate futures;
extern crate libc;
extern crate rand;
extern crate spdk_rs;
extern crate utils_rustfs;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate colored;

pub mod language;

fn main() {
    language::main();
}
