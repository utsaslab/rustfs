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
use std::mem;

async fn run(poller: spdk_rs::io_channel::PollerHandle) {
    match await!(run_inner()) {
            Ok(_) => println!("Successful"),
            Err(err) => println!("Failure: {:?}", err),
    }
    // FIXME: drop poller can lead to seg fault
    //drop(poller);
    spdk_rs::event::app_stop(true);
}

async fn run_inner() -> Result<(), Error> {

    let mut first_bdev = spdk_rs::bdev::first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = spdk_rs::bdev::next(&bdev);
    }

    let ret = spdk_rs::bdev::get_by_name("Malloc0");
    let bdev = ret.unwrap();
    let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();

    match spdk_rs::bdev::open(bdev.clone(), true, &mut desc) {
        Ok(_) => println!("Successfully open the device"),
        _ => {}
    }

    let io_channel = spdk_rs::bdev::get_io_channel(desc.clone())?;

    let blk_size = spdk_rs::bdev::get_block_size(bdev.clone());
    println!("blk_size: {}", blk_size);

    let buf_align = spdk_rs::bdev::get_buf_align(bdev.clone());
    println!("buf_align: {}", buf_align);

    let mut write_buf = spdk_rs::env::dma_zmalloc(blk_size as usize, buf_align);

    write_buf.fill(blk_size as usize, "%s\n", "Hello world!");

    match await!(spdk_rs::bdev::write(desc.clone(), &io_channel, &write_buf, 0, blk_size as u64)) {
        Ok(_) => println!("Successfully write to bdev"),
        _ => {}
    }

    let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size as usize, buf_align);
    
    match await!(spdk_rs::bdev::read(desc.clone(), &io_channel, &mut read_buf, 0, blk_size as u64)) {
        Ok(_) => println!("Successfully read from bdev"),
        _ => {}
    }

    println!("Read string from bdev: {}", read_buf.read());

    drop(read_buf);
    drop(write_buf);

    spdk_rs::thread::put_io_channel(io_channel);
    spdk_rs::bdev::close(desc);
    spdk_rs::event::app_stop(true);
    Ok(())
}

fn main()
{
    println!("Rust binary path: {}", env::current_exe().unwrap().to_str().unwrap());
    let config_file = Path::new("bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("hello_bdev");
    opts.config_file(config_file.to_str().unwrap());

    let _ret = opts.start(|| {
        let executor = spdk_rs::executor::initialize();
        mem::forget(executor);

        let poller = spdk_rs::io_channel::poller_register(spdk_rs::executor::pure_poll);
        spdk_rs::executor::spawn(run(poller));
    });
    println!("Successfully shutdown SPDK framework");
}
