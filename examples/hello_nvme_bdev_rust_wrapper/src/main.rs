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

use spdk_rs::{spdk_app_stop, AppContext, SpdkAppOpts, SpdkBdev, SpdkBdevIO,
              SpdkBdevIoCompletionCb};
use std::ffi::c_void;
use std::ptr;
use std::rc::Rc;
use std::cell::RefCell;

use futures::future::Future;
use futures::executor::block_on;
use futures::FutureExt;
use tokio_core::reactor::Core;

fn write_complete(spdkBdevIo: &mut SpdkBdevIO, success: &mut bool, cb_arg: &mut AppContext) {
    println!("Get to the write_complete");
    println!("success: {}", success);
    println!("name: {}", cb_arg.bdev_name());
    spdk_app_stop(true);
}

fn hello_start(context: &mut AppContext) {
    println!("Successfully started the application");
    let mut first_bdev = SpdkBdev::spdk_bdev_first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = SpdkBdev::spdk_bdev_next(&bdev);
    }
    match context.set_bdev() {
        Err(_e) => {
            println!("{}", _e.to_owned());
            spdk_app_stop(false);
        }
        Ok(_) => ()
    };
    match context.spdk_bdev_open(true) {
        Err(_e) => {
            println!("{}", _e.to_owned());
            spdk_app_stop(false);
        }
        Ok(_) => ()
    }
    match context.spdk_bdev_get_io_channel() {
        Err(_e) => {
            println!("{}", _e.to_owned());
            context.spdk_bdev_close();
            spdk_app_stop(false);
        }
        Ok(_) => ()
    }
    match context.allocate_buff() {
        Err(_e) => {
            println!("{}", _e.to_owned());
            context.spdk_bdev_put_io_channel();
            context.spdk_bdev_close();
            spdk_app_stop(false);
        }
        Ok(_) => ()
    }
    context.write_buff("Hello World!");

    println!("Writing to the bdev");
    let blk_size = SpdkBdev::spdk_bdev_get_block_size(context.bdev().unwrap());
    let mut spdk_bdev_io: SpdkBdevIO;
//    SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
//                                    context.bdev_io_channel(),
//                                    context.buff(),
//                                    0,
//                                    blk_size as u64).then(|result| {
//        match result {
//            Ok(bdev_io) => spdk_bdev_io = bdev_io,
//            Err(_e) => spdk_bdev_io = SpdkBdevIO::new()
//        }
//    });
    println!("92");
//    let future = SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
//                                           context.bdev_io_channel(),
//                                           context.buff(),
//                                           0,
//                                           blk_size as u64);

//    let future = SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
//                                           context.bdev_io_channel(),
//                                           context.buff(),
//                                           0,
//                                           blk_size as u64);

//    match block_on(future) {
//        Ok(bdev_io) => spdk_bdev_io = bdev_io,
//        Err(_e) => {}
//    }
    futures::executor::block_on(
            SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
                                           context.bdev_io_channel(),
                                           context.buff(),
                                           0,
                                           blk_size as u64)
    );
//    block_on(future).unwrap();
    println!("Get to the write_complete");

//    let mut spdk_bdev_io: SpdkBdevIO = SpdkBdevIO::new();
//    let mut success: bool = false;
//    let mut context_cpy = context.clone();
//    match context_cpy.spdk_bdev_write(0, ||{
//        write_complete(&mut spdk_bdev_io, &mut success,  context)
//    }) {
//        Err(_e) => {
//            println!("{}", _e.to_owned());
//            context.spdk_bdev_close();
//            context.spdk_bdev_put_io_channel();
//            spdk_app_stop(false);
//        }
//        Ok(_) => ()
//    }
//    println!("context bdev_name: {}", context.bdev_name());
    context.spdk_bdev_close();
    spdk_app_stop(true);
}

async fn print_async() {
    println!("Hello from print_async");
}

async fn print_async2() -> Result<i32, String> {
    Ok(10)
}

fn main()
{
    // Some test code to be deleted.
    let future = print_async();
    let future2 = print_async2();
    println!("Hello from main");
    block_on(future);
    println!("{}", block_on(future2).unwrap());

    println!("Enter main");
    let mut opts = SpdkAppOpts::new();
    opts.name("hello_bdev");
    opts.config_file("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf");

    let mut context = AppContext::new();
    context.set_bdev_name("Nvme0n1");

    let ret = opts.start(|| {
        hello_start(&mut context);
    });
}
