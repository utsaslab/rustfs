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
#![feature(generators)]
#![feature(futures_api)]

#[macro_use]
extern crate failure;
extern crate spdk_rs;

extern crate futures;
#[macro_use]
extern crate tokio;

extern crate tokio_async_await;

use std::path::Path;
use failure::Error;
use std::env;

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

use std::future::{Future as StdFuture};
use tokio::runtime::Runtime;
use tokio::prelude::*;

async fn map_ok<T: StdFuture>(future: T) -> Result<(),()> {
    let _ = await!(future);
    Ok(())
}

pub fn run_spdk<F>(future: F)
where F: StdFuture<Output = ()> + Send + 'static,
{
    use tokio_async_await::compat::backward;
    let future = backward::Compat::new(map_ok(future));

    let mut rt = Runtime::new().unwrap();
    rt.block_on(future);
    rt.shutdown_now().wait().unwrap();
    spdk_rs::event::app_stop(true);
}

async fn run() {
    match await!(run_inner()) {
            Ok(_) => println!("Successful"),
            Err(err) => println!("Failure: {:?}", err),
    }
    // FIXME: it's very strange that we must call app_stop twice (here and in `run_spdk`)
    // to stop the SPDK framework
    spdk_rs::event::app_stop(true);
}

async fn run_inner() -> Result<(), Error> {
    let mut first_bdev = spdk_rs::bdev::first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = spdk_rs::bdev::next(&bdev);
    }

    let mut ret = spdk_rs::bdev::get_by_name("Malloc0");
    let bdev = ret.unwrap();
    let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();

    let ret = spdk_rs::bdev::open(bdev.clone(), true, &mut desc);
    match ret {
        Ok(_) => println!("Successfully open the device"),
        _ => {}
    }

    let ret = spdk_rs::thread::allocate_thread("new_thread");
    match ret {
        Ok(_) => println!("Successfully allocate a thread"),
        _ => {}
    }

    let mut io_channel = spdk_rs::bdev::get_io_channel(desc.clone());
    match ret {
        Ok(_) => println!("Successfully create a bdev I/O channel"),
        _ => {}
    }

    let blk_size = spdk_rs::bdev::spdk_bdev_get_block_size(bdev);
    println!("blk_size: {}", blk_size);

    spdk_rs::bdev::close(desc);
    spdk_rs::thread::free_thread();
    Ok(())
}

fn main()
{
    println!("Rust binary path: {}", env::current_exe().unwrap().to_str().unwrap());
    let config_file = Path::new("bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("hello_bdev");
    opts.config_file(config_file.to_str().unwrap());

    let ret = opts.start(|| {
        //NOTE: Alternatively, we can use `tokio::run_async(run())` but doing so requires us to use
        // C-c to terminate program
        run_spdk(run());
    });
    println!("Successfully shutdown SPDK framework");
}
