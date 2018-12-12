/*************************************************************************
  > File Name:       main.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    09/07/18
  > Description:

    This program performs the same functionality as "hello_nvme_bdev.c".
    It uses the spdk-rs rust-friendly FFI.

 ************************************************************************/
#![feature(nll)]
#![feature(await_macro, async_await)]
#![feature(proc_macro, generators)]
#![feature(futures_api)]

#[macro_use]
extern crate failure;
extern crate spdk_rs;

extern crate futures;
#[macro_use]
extern crate tokio;

use std::path::Path;
use failure::Error;

//use std::ffi::c_void;
//use std::ptr;
//use std::rc::Rc;
//use std::cell::RefCell;
//use std::mem;
//
//use futures::future::Future;
//use futures::executor::block_on;
//use futures::FutureExt;
//use tokio_core::reactor::Core;

async fn run() {
    match await!(run_inner()) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err),
    }

    spdk_rs::event::app_stop(true);
}

async fn run_inner() -> Result<(), Error> {
    let mut first_bdev = spdk_rs::bdev::first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = spdk_rs::bdev::next(&bdev);
    }

    let mut bdev = spdk_rs::bdev::get_by_name("Malloc0");
    let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();
    let ret = spdk_rs::bdev::open(&bdev.unwrap(), true, &mut desc);
    match ret {
        Ok(_) => println!("Successfully open the device"),
        _ => {}
    }
    if desc.to_raw().is_null() {
        println!("Error");
    }
    let mut io_channel = spdk_rs::bdev::get_io_channel(desc.clone());

    spdk_rs::bdev::close(desc);

    Ok(())
}

fn main()
{
    println!("Enter main");
    let config_file = Path::new("bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("hello_bdev");
    opts.config_file(config_file.to_str().unwrap());

    let ret = opts.start(|| {
        tokio::run_async(run());
    });
}
