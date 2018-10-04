/*************************************************************************
  > File Name:       main.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    09/07/18
  > Description:

    This program performs the same functionality as "hello_nvme_bdev.c".
    It uses the spdk-rs rust-friendly FFI.

 ************************************************************************/

extern crate spdk_rs;

use spdk_rs::{AppOpts, AppContext};

fn main()
{
    println!("Enter main");
    let mut opts = AppOpts::new();
    opts.name("hello_bdev");
    opts.config_file("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf");

    let mut context =AppContext::new();
    context.bdev_name("Nvme0n1");

//
//    let ret = opts.start(|| {
//        let executor = executor::initialize();
//
//        // TODO: fixup
//        mem::forget(executor);
//
//        // Register the executor poller
//        let poller = io_channel::poller_register(executor::pure_poll);
//
//        executor::spawn(run(poller));
//    });
}
