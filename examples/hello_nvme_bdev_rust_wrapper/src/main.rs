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

use futures::future::Future;
use futures::executor::block_on;
use futures::FutureExt;
use tokio_core::reactor::Core;
use failure::Error;

use std::path::Path;
use std::mem;


//fn write_complete(spdkBdevIo: &mut SpdkBdevIO, success: &mut bool, cb_arg: &mut AppContext) {
//    println!("Get to the write_complete");
//    println!("success: {}", success);
//    println!("name: {}", cb_arg.bdev_name());
//    spdk_app_stop(true);
//}
//
//fn hello_start(context: &mut AppContext) {
//    println!("Successfully started the application");
//    let mut first_bdev = SpdkBdev::spdk_bdev_first();
//    while !first_bdev.is_none() {
//        let bdev = first_bdev.unwrap();
//        println!("bdev name: {}", bdev.name());
//        first_bdev = SpdkBdev::spdk_bdev_next(&bdev);
//    }
//    match context.set_bdev() {
//        Err(_e) => {
//            println!("{}", _e.to_owned());
//            spdk_app_stop(false);
//        }
//        Ok(_) => ()
//    };
//    match context.spdk_bdev_open(true) {
//        Err(_e) => {
//            println!("{}", _e.to_owned());
//            spdk_app_stop(false);
//        }
//        Ok(_) => ()
//    }
//    match context.spdk_bdev_get_io_channel() {
//        Err(_e) => {
//            println!("{}", _e.to_owned());
//            context.spdk_bdev_close();
//            spdk_app_stop(false);
//        }
//        Ok(_) => ()
//    }
//    match context.allocate_buff() {
//        Err(_e) => {
//            println!("{}", _e.to_owned());
//            context.spdk_bdev_put_io_channel();
//            context.spdk_bdev_close();
//            spdk_app_stop(false);
//        }
//        Ok(_) => ()
//    }
//    context.write_buff("Hello World!");
//
//    println!("Writing to the bdev");
//    let blk_size = SpdkBdev::spdk_bdev_get_block_size(context.bdev().unwrap());
//    let mut spdk_bdev_io: SpdkBdevIO;
////    SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
////                                    context.bdev_io_channel(),
////                                    context.buff(),
////                                    0,
////                                    blk_size as u64).then(|result| {
////        match result {
////            Ok(bdev_io) => spdk_bdev_io = bdev_io,
////            Err(_e) => spdk_bdev_io = SpdkBdevIO::new()
////        }
////    });
//    println!("92");
////    let future = SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
////                                           context.bdev_io_channel(),
////                                           context.buff(),
////                                           0,
////                                           blk_size as u64);
//
////    let future = SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
////                                           context.bdev_io_channel(),
////                                           context.buff(),
////                                           0,
////                                           blk_size as u64);
//
////    match block_on(future) {
////        Ok(bdev_io) => spdk_bdev_io = bdev_io,
////        Err(_e) => {}
////    }
//    futures::executor::block_on(
//            SpdkBdev::spdk_bdev_write(context.bdev_desc().unwrap(),
//                                           context.bdev_io_channel(),
//                                           context.buff(),
//                                           0,
//                                           blk_size as u64)
//    );
////    block_on(future).unwrap();
//    println!("Get to the write_complete");
//
////    let mut spdk_bdev_io: SpdkBdevIO = SpdkBdevIO::new();
////    let mut success: bool = false;
////    let mut context_cpy = context.clone();
////    match context_cpy.spdk_bdev_write(0, ||{
////        write_complete(&mut spdk_bdev_io, &mut success,  context)
////    }) {
////        Err(_e) => {
////            println!("{}", _e.to_owned());
////            context.spdk_bdev_close();
////            context.spdk_bdev_put_io_channel();
////            spdk_app_stop(false);
////        }
////        Ok(_) => ()
////    }
////    println!("context bdev_name: {}", context.bdev_name());
//    context.spdk_bdev_close();
//    spdk_app_stop(true);
//}

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
