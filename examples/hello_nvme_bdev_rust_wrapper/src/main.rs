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

use spdk_rs::{AppOpts, AppContext, app_stop, Bdev};

// https://stackoverflow.com/questions/33376486/is-there-a-way-other-than-traits-to-add-methods-to-a-type-i-dont-own
trait AppContextExt {
    fn hello_start(context : &AppContext);
}

impl AppContextExt for AppContext {
    fn hello_start(context : &AppContext) {
        println!("Successfully started the application");
        let mut first_bdev = Bdev::spdk_bdev_first();
        while  !first_bdev.is_none() {
            let mut bdev = first_bdev.unwrap();
            println!("bdev name: {}", bdev.name());
            first_bdev = Bdev::spdk_bdev_next(&bdev);
        }
        let bdev_name = context.get_bdev_name();
        let bdev = Bdev::spdk_bdev_get_by_name(bdev_name);
        match bdev {
            Err(E) => {
                let s = E.to_owned();
                let s_slice = &s[..];
                println!("{}", E);
                app_stop(false);
            }
            Ok(T) => {}
        }
        app_stop(true);
    }
}

fn main()
{
    println!("Enter main");
    let mut opts = AppOpts::new();
    opts.name("hello_bdev");
    opts.config_file("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf");

    let mut context = AppContext::new();
    context.bdev_name("Nvme0n1");

    let ret = opts.start(|| {
        spdk_rs::AppContext::hello_start(&context);
    }, &context);
}
