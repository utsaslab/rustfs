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

extern crate spdk_rs;

use spdk_rs::{AppOpts, AppContext, app_stop, Bdev};


fn hello_start(context: &mut AppContext) {
    println!("Successfully started the application");
    let mut first_bdev = Bdev::spdk_bdev_first();
    while !first_bdev.is_none() {
        let bdev = first_bdev.unwrap();
        println!("bdev name: {}", bdev.name());
        first_bdev = Bdev::spdk_bdev_next(&bdev);
    }
    match context.set_bdev(){
        Err(_e) => {
            println!("{}", _e.to_owned());
            app_stop(false);
        },
        Ok(_) => ()
    };
    match context.spdk_bdev_open(true) {
        Err(_e) => {
            println!("{}", _e.to_owned());
            app_stop(false);
        }
        Ok(_) => ()
    }
    context.spdk_bdev_close();
    app_stop(true);
}

fn main()
{
    println!("Enter main");
    let mut opts = AppOpts::new();
    opts.name("hello_bdev");
    opts.config_file("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf");

    let mut context = AppContext::new();
    context.set_bdev_name("Nvme0n1");

    let ret = opts.start(|| {
        hello_start(&mut context);
    });
}
