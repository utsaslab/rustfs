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

extern "C" fn read_complete(bdev_io: *mut spdk_bdev_io,
                            success: bool,
                            cb_arg: *mut c_void) {
    let hello_context: *mut hello_context_t = cb_arg as *mut hello_context_t;

    unsafe {
        match success {
            true => println!("Read string from bdev: {}", CStr::from_ptr((*hello_context).buff).to_str().unwrap()),
            false => println!("bdev io read error")
        }

        spdk_bdev_free_io(bdev_io);
        spdk_put_io_channel((*hello_context).bdev_io_channel);
        spdk_bdev_close((*hello_context).bdev_desc);
        println!("Stopping app");
        spdk_app_stop(if success { 0 } else { -1 });
    }
}


extern "C" fn write_complete(bdev_io: *mut spdk_bdev_io,
                             success: bool,
                             cb_arg: *mut c_void) {
    let hello_context: *mut hello_context_t = cb_arg as *mut hello_context_t;
    let rc: c_int;
    let blk_size: u32;

    unsafe {
        spdk_bdev_free_io(bdev_io);

        match success {
            true => println!("bdev io write completed successfully"),
            false => {
                println!("bdev io write error: {}", EIO);
                spdk_put_io_channel((*hello_context).bdev_io_channel);
                spdk_bdev_close((*hello_context).bdev_desc);
                spdk_app_stop(-1);
                return;
            }
        }

        blk_size = spdk_bdev_get_block_size((*hello_context).bdev);
        memset((*hello_context).buff as *mut c_void, 0, blk_size as usize);

        println!("Reading io");
        let hello_context_ptr: *mut c_void = hello_context as *mut _ as *mut c_void;
        rc = spdk_bdev_read((*hello_context).bdev_desc,
                            (*hello_context).bdev_io_channel,
                            (*hello_context).buff as *mut c_void,
                            0,
                            blk_size as u64,
                            Some(read_complete),
                            hello_context_ptr);
        if rc != 0 {
            println!("{} error while reading from bdev: {}", CStr::from_ptr(spdk_strerror(-rc)).to_str().unwrap(), rc);
            spdk_put_io_channel((*hello_context).bdev_io_channel);
            spdk_bdev_close((*hello_context).bdev_desc);
            spdk_app_stop(-1);
            return;
        }
    }
}

extern "C" fn hello_start(_arg1: *mut c_void, _arg2: *mut c_void) {
    let hello_context: *mut hello_context_t = _arg1 as *mut hello_context_t;
    let blk_size: u32;
    let buf_align: usize;
    let mut rc: c_int;
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

        (*hello_context).bdev = spdk_bdev_get_by_name((*hello_context).bdev_name);
        if (*hello_context).bdev.is_null() {
            println!("Could not find the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
            spdk_app_stop(-1);
            return;
        }

        println!("Opening the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
        rc = spdk_bdev_open((*hello_context).bdev, true, None, ptr::null_mut(), &mut (*hello_context).bdev_desc);
        if rc != 0 {
            println!("Could not open bdev: {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
            spdk_app_stop(-1);
            return;
        }

        println!("Opening io channel");
        (*hello_context).bdev_io_channel = spdk_bdev_get_io_channel((*hello_context).bdev_desc);
        if (*hello_context).bdev_io_channel.is_null() {
            println!("Could not create bdev I/O channel!!");
            spdk_bdev_close((*hello_context).bdev_desc);
            spdk_app_stop(-1);
            return;
        }

        blk_size = spdk_bdev_get_block_size((*hello_context).bdev);
        buf_align = spdk_bdev_get_buf_align((*hello_context).bdev);
        (*hello_context).buff = spdk_dma_zmalloc(blk_size as usize, buf_align, ptr::null_mut()) as *mut c_char;
        if (*hello_context).buff.is_null() {
            println!("Failed to allocate buffer");
            spdk_put_io_channel((*hello_context).bdev_io_channel);
            spdk_bdev_close((*hello_context).bdev_desc);
            spdk_app_stop(-1);
            return;
        }

        let owned_fmt = CString::new("%s\n").unwrap();
        let fmt: *const c_char = owned_fmt.as_ptr();
        libc::snprintf((*hello_context).buff, blk_size as usize, fmt, "Hello World!\n");

        println!("Writing to the bdev");
        let hello_context_ptr: *mut c_void = hello_context as *mut _ as *mut c_void;
        rc = spdk_bdev_write((*hello_context).bdev_desc, (*hello_context).bdev_io_channel,
                             (*hello_context).buff as *mut c_void, 0, blk_size as u64, Some(write_complete), hello_context_ptr);
        if rc != 0 {
            println!("{0} error while writing to bdev: {1}", CStr::from_ptr(spdk_strerror(-rc)).to_str().unwrap(), rc);
            spdk_bdev_close((*hello_context).bdev_desc);
            spdk_put_io_channel((*hello_context).bdev_io_channel);
            spdk_app_stop(-1);
            return;
        }
    }
}

fn main() {
    println!("Enter main");

    unsafe {
        let mut opts: spdk_app_opts;
        opts = std::mem::uninitialized();
        spdk_app_opts_init(&mut opts);

        let mut hello_context: hello_context_t = std::mem::uninitialized();

        let owned_name = CString::new("hello_bdev").unwrap();
        opts.name = owned_name.as_ptr();

        let owned_config_file = CString::new("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf").unwrap();
        opts.config_file = owned_config_file.as_ptr();

        let owned_bdev_name = CString::new("Nvme0n1").unwrap();
        hello_context.bdev_name = owned_bdev_name.as_ptr();

        let hello_context_ptr: *mut c_void = &mut hello_context as *mut _ as *mut c_void;
        let rc: c_int = spdk_app_start(&mut opts, Some(hello_start), hello_context_ptr, ptr::null_mut());
        if rc != 0 {
            panic!("ERROR starting application");
        }

        spdk_dma_free(hello_context.buff as *mut c_void);

        spdk_app_fini();
    }
}
