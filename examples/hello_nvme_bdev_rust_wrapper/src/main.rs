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

extern crate futures;
extern crate spdk_rs;
extern crate tokio_core;
#[macro_use]
extern crate failure;

use spdk_rs::{AppContext, SpdkAppOpts, SpdkBdev, SpdkBdevIO,
              SpdkBdevIoCompletionCb, executor, io_channel};
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;
use std::path::Path;
use std::mem;

use futures::future::Future;
use futures::executor::block_on;
use futures::FutureExt;
use tokio_core::reactor::Core;
use failure::Error;


async fn run(poller: io_channel::PollerHandle) {
    match await!(run_inner()) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err),
    }

    drop(poller);

    spdk_rs::event::app_stop(true);
}

async fn run_inner() -> Result<(), Error> {

    // let mut bdev = spdk_rs::bdev::get_by_name("Malloc0");

    // let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();

    // let ret = spdk_rs::bdev::open(&bdev.unwrap(), true, &mut desc);

    // spdk_rs::bdev::close(desc);
    
    let mut first_bdev = spdk_rs::bdev::first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = spdk_rs::bdev::next(&bdev);

    }

    Ok(())
}

fn main()
{
    println!("Enter main");
    let config_file = Path::new("bdev.conf").canonicalize().unwrap();
    let mut opts = SpdkAppOpts::new();

    opts.name("hello_bdev");
    opts.config_file(config_file.to_str().unwrap());

    let ret = opts.start(|| {
       let executor = executor::initialize();

        // TODO: fixup
        mem::forget(executor);

        // Register the executor poller
        let poller = io_channel::poller_register(executor::pure_poll);

        executor::spawn(run(poller));
    });
}
