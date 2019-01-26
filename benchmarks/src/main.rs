/*************************************************************************
 > File Name:       main.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    12/15/18
 > Description:

   Main driver program to run various benchmarks
************************************************************************/
#![feature(await_macro, async_await, futures_api, nll, generators)]
#![feature(core_intrinsics)]

extern crate failure;
extern crate futures;
extern crate libc;
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
