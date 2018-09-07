#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

extern crate libc;

//    use std::io::prelude::*;
use std::ffi::{CString, CStr};
use std::ptr;
use std::os::raw::{c_void, c_char, c_int};

#[derive(Debug)]
struct hello_context_t {
    bdev: *mut spdk_bdev,
    bdev_desc: *mut spdk_bdev_desc,
    bdev_io_channel: *mut spdk_io_channel,
    buff: *mut c_char,
    bdev_name: *const c_char,
}

fn hello_bdev_usage() {
    println!(" -b <bdev>                 name of the bdev to use");
}

extern "C" fn hello_start(_arg1: *mut c_void, _arg2: *mut c_void) {
    let hello_context: *mut hello_context_t = unsafe { _arg1 as *mut hello_context_t };
    let mut blk_size: u32;
    let mut buf_align: u32;
    let mut rc: c_int = 0;
    unsafe { (*hello_context).bdev = ptr::null_mut(); }
    unsafe { (*hello_context).bdev_desc = ptr::null_mut(); }

    println!("Successfully started the application");

    unsafe {
        println!("Try to get a list of bdev ... ");
        let mut first: *mut spdk_bdev = spdk_bdev_first();
        while !first.is_null() {
            let owned_fmt = CString::new("bdev name: %s\n").unwrap();
            let fmt: *const c_char = owned_fmt.as_ptr();
            libc::printf(fmt, (*first).name);
            first = spdk_bdev_next(first);
        }
    }

    unsafe {
        (*hello_context).bdev = spdk_bdev_get_by_name((*hello_context).bdev_name);
        if (*hello_context).bdev.is_null() {
            println!("Could not find the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
            spdk_app_stop(-1);
            return;
        }
    }

    unsafe {
        println!("Opening the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
//        let ptr : *mut *mut spdk_bdev_desc = ;
        rc = spdk_bdev_open((*hello_context).bdev, true, None, ptr::null_mut(), &mut (*hello_context).bdev_desc);
        if rc != 0 {
            println!("Could not open bdev: {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
            spdk_app_stop(-1);
            return
        }
    }

    unsafe {
        println!("Opening io channel");
        (*hello_context).bdev_io_channel = spdk_bdev_get_io_channel((*hello_context).bdev_desc);
        if (*hello_context).bdev_io_channel.is_null() {
            println!("Could not create bdev I/O channel!!");
            spdk_bdev_close((*hello_context).bdev_desc);
            spdk_app_stop(-1);
            return;
        }
    }

//
//            blk_size = spdk_bdev_get_block_size(hello_context.bdev);
//            buf_align = spdk_bdev_get_buf_align(hello_context.bdev);
//            hello_context.buff = match spdk_dma_zmalloc(blk_size, buf_align, None) {
//                Some(val) => val,
//                None => {
//                    println!("Failed to allocate buffer");
//                    spdk_put_io_channel(hello_context.bdev_io_channel);
//                    spdk_bdev_close(hello_context.bdev_desc);
//                    spdk_app_stop(-1);
//                    return;
//                }
//            };
//
//            match hello_context.buff.write("Hello World!") {
//                OK(val) => {}
//                Err(_e) => {
//                    println!("Error in writing message!");
//                    spdk_bdev_close(hello_context.bdev_desc);
//                    spdk_app_stop(-1);
//                    return;
//                }
//            };
//
//            println!("Writing to the bdev");
//            rc = match spdk_bdev_write(hello_context.bdev_desc, hello_context.bdev_io_channel,
//                                       hello_context.buff, 0, blk_size, write_complete, hello_context) {
//                1 => {
//                    panic!("{0} error while writing to bdev: {1}", spdk_sterror(-rc), rc);
//                    spdk_bdev_close(hello_context.bdev_desc);
//                    spdk_put_io_channel(hello_context.bdev_io_channel);
//                    spdk_app_stop(-1);
//                    return;
//                }
//            };
}

fn main() {


//        fn read_complete(bdev_io: *mut spdk_bdev_io,
//                         success: bool,
//                         cb_arg: hello_context_t) {
//            let mut hello_context: hello_context_t = cb_arg;
//
//            match success {
//                true => println!("Read string from bdev: {}", hello_context.buff);
//                false => println!("bdev io read error");
//            }
//
//            spdk_bdev_free_io(bdev_io);
//            spdk_put_io_channel(hello_context.bdev_io_channel);
//            spdk_bdev_close(hello_context.bdev_desc);
//            println!("Stopping app");
//            spdk_app_stop(if success { 0 } else { -1 });
//        }
//
//        fn write_complete(bdev_io: *mut spdk_bdev_io,
//                          success: bool,
//                          cb_arg: hello_context_t) {
//            // hzy: If the variable doesn't interact with the bindings, we should strive to
//            // use native Rust.
//            let mut hello_context: hello_context_t = cb_arg;
//            let mut rc: std::os::raw::c_int;
//            let mut blk_size: u32;
//
//            spdk_bdev_free_io(bdev_io);
//
//            match success {
//                true => println!("bdev io write completed successfully"),
//                false => {
//                    println!("bdev io write error: {}", EIO);
//                    spdk_put_io_channel(hello_context.bdev_io_channel);
//                    spdk_bdev_close(hello_context.bdev_desc);
//                    spdk_app_stop(-1);
//                    return;
//                }
//            }
//
//            blk_size = spdk_bdev_get_block_size(hello_context.bdev);
//            memset(hello_context.buff, 0, blk_size);
//
//            println!("Reading io");
//            rc = spdk_bdev_read(hello_context.bdev_desc,
//                                hello_context.bdev_io_channel,
//                                hello_context.buff,
//                                0,
//                                blk_size,
//                                read_complete,
//                                hello_context);
//            if rc {
//                println!("{} error while reading from bdev: {}", spdk_strerror(-rc), rc);
//                spdk_put_io_channel(hello_context.bdev_io_channel);
//                spdk_bdev_close(hello_context.bdev_desc);
//                spdk_app_stop(-1);
//                return;
//            }
//        }

    println!("Enter main");

    let mut opts: spdk_app_opts;
    unsafe {
        opts = std::mem::uninitialized();
        spdk_app_opts_init(&mut opts);
    }

    let mut hello_context: hello_context_t = unsafe { std::mem::uninitialized() };

    let owned_name = CString::new("hello_bdev").unwrap();
    opts.name = owned_name.as_ptr();

    let owned_config_file = CString::new("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf").unwrap();
    opts.config_file = owned_config_file.as_ptr();

    let owned_bdev_name = CString::new("Nvme0n1").unwrap();
    hello_context.bdev_name = owned_bdev_name.as_ptr();

    let hello_context_ptr: *mut c_void = &mut hello_context as *mut _ as *mut c_void;
    let rc: c_int = unsafe { spdk_app_start(&mut opts, Some(hello_start), hello_context_ptr, ptr::null_mut()) };
    if rc != 0 {
        panic!("ERROR starting application");
    }

    // hzy: here we still call free method, which still looks like C. Doing this
    // doesn't utilize the Rust ownership feature.
//            spdk_dma_free(hello_context.buff);

    unsafe { spdk_app_fini() };
}
