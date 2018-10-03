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

use spdk_rs::AppOpts;
use std::path::Path;

fn main()
{
    println!("Enter main");
    let config_file = Path::new("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf").canonicalize().unwrap();
    let mut opts = AppOpts::new();

//    opts.name("hello_blob");
//    opts.config_file(config_file.to_str().unwrap());
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
