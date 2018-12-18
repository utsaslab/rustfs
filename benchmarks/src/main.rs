/*************************************************************************
  > File Name:       main.rs.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    12/15/18
  > Description:
    
    Main driver program to run various benchmarks
 ************************************************************************/
#![feature(await_macro, async_await, futures_api, nll, generators)]

extern crate failure;
extern crate spdk_rs;
extern crate futures;
extern crate utils_rustfs;

pub mod language;

fn main() {
    language::main();
}