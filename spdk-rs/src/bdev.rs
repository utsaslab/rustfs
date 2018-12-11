/*************************************************************************
  > File Name:       bdev.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    9/16/18
  > Description:
    
    FFI for "bdev.h"

    Note that not all functions belong to "bdev.h" are implemented here.
    For example, spdk_bdev_open() is implemented in the context instead
    because spdk_bdev_open works with struct spdk_bdev* and
    struct spdk_bdev_desc**, which usually used with the context struct.
 ************************************************************************/

use crate::raw;
use crate::SpdkBdevIO;
use crate::env;

use std::ffi::{CString, CStr, c_void};
use std::marker;
use std::ptr;

use failure::Error;

use futures::channel::oneshot::Sender;
use futures::channel::oneshot;
use futures::channel::mpsc;
use futures::executor::block_on;

#[derive(Debug, Fail)]
pub enum BdevError {
    #[fail(
    display = "Error in write completion({}): {}, offset: {}, length: {}",
    _0,
    _1,
    _2,
    _3
    )]
    WriteError(String, i32, u64, u64),

    #[fail(display = "Could not find a bdev: {}", _0)]
    NotFound(String),

    #[fail(display = "Could not open device: {}", _0)]
    OpenError(String),
}

pub struct SpdkBdev {
    raw: *mut raw::spdk_bdev,
}

pub fn get_by_name<S>(bdev_name: S) -> Result<SpdkBdev, Error>
where
    S: Into<String> + Clone,
{
    let name_cstring = CString::new(bdev_name.clone().into()).expect("Couldn't create a string");

    let bdev = unsafe {
        raw::spdk_bdev_get_by_name(name_cstring.as_ptr())
    };
    if bdev.is_null() {
        return Err(BdevError::NotFound(bdev_name.clone().into()))?;
    }

    Ok(SpdkBdev::from_raw(bdev))
}

pub fn open(bdev: &SpdkBdev, write: bool, bdev_desc: &mut SpdkBdevDesc) -> Result<(), Error> {
    unsafe {
        let rc = raw::spdk_bdev_open(bdev.to_raw(), write, None, ptr::null_mut(), bdev_desc.mut_to_raw());
        match rc != 0 {
            true => {
                Err(BdevError::OpenError(bdev.name().to_string()))?
            }
            false => {
                Ok(())
            }
        }
    }
}

pub fn close(bdev_desc: SpdkBdevDesc) {
    unsafe {
        raw::spdk_bdev_close(bdev_desc.to_raw())
    }
}


pub fn first() -> Option<SpdkBdev> {
    unsafe {
        let ptr = raw::spdk_bdev_first();
        if ptr.is_null() {
            None
        } else {
            Some(SpdkBdev::from_raw(ptr))
        }
    }
}

pub fn next(prev: &SpdkBdev) -> Option<SpdkBdev> {
    unsafe {
        let ptr = raw::spdk_bdev_next(prev.raw);
        if ptr.is_null() {
            None
        } else {
            Some(SpdkBdev::from_raw(ptr))
        }
    }
}

impl SpdkBdev {
    pub fn from_raw(raw: *mut raw::spdk_bdev) -> SpdkBdev {
        unsafe {
            SpdkBdev {
                raw: raw,
            }
        }
    }








//    pub async fn spdk_bdev_write(desc: SpdkBdevDesc,
//                                 ch: SpdkIoChannel,
//                                 buf: Buf,
//                                 offset: u64,
//                                 nbytes: u64) -> Result<SpdkBdevIO, Error> {
//        let (sender, receiver) = oneshot::channel();
//        print!("111");
//        let ret: i32;
//        unsafe {
//            ret = raw::spdk_bdev_write(
//                desc.raw,
//                ch.raw,
//                buf.to_raw(),
//                offset,
//                nbytes,
//                Some(spdk_bdev_io_completion_cb),
//                cb_arg::<*mut raw::spdk_bdev_io>(sender), // TODO: we probably need to modify here to take in closure again as we need to indicate when callback need to call spdk_app_stop()
//            );
//        };
//        // TODO: we probably need to handle the case where ret != 0
//        let res = await!(receiver).expect("Cancellation is not supported");
//
//        match res {
//            Ok(bdev_io) => Ok(
//                SpdkBdevIO::from_raw(bdev_io)
//            ),
//            Err(_e) => Err(BdevError::WriteError(
//                desc.spdk_bdev_desc_get_bdev().name().to_string(),
//                -1,
//                offset,
//                nbytes,
//            ))?,
//        }
//    }

    pub async fn spdk_bdev_write(desc: SpdkBdevDesc,
                                 ch: SpdkIoChannel,
                                 buf: env::Buf,
                                 offset: u64,
                                 nbytes: u64) -> Result<SpdkBdevIO, Error> {
        let (sender, receiver) = oneshot::channel();
        print!("111");
        let ret: i32;
        unsafe {
            ret = raw::spdk_bdev_write(
                desc.raw,
                ch.raw,
                buf.to_raw(),
                offset,
                nbytes,
                Some(spdk_bdev_io_completion_cb),
                cb_arg::<*mut raw::spdk_bdev_io>(sender), // TODO: we probably need to modify here to take in closure again as we need to indicate when callback need to call spdk_app_stop()
            );
        };
        // TODO: we probably need to handle the case where ret != 0
        let res = await!(receiver).expect("Cancellation is not supported");

        match res {
            Ok(bdev_io) => Ok(
                SpdkBdevIO::from_raw(bdev_io)
            ),
            Err(_e) => Err(BdevError::WriteError(
                desc.spdk_bdev_desc_get_bdev().name().to_string(),
                -1,
                offset,
                nbytes,
            ))?,
        }
    }

//    pub fn spdk_bdev_write(desc: SpdkBdevDesc,
//                                 ch: SpdkIoChannel,
//                                 buf: Buf,
//                                 offset: u64,
//                                 nbytes: u64) -> () {
//        print!("111");
//        let ret: i32;
//        unsafe {
//            ret = raw::spdk_bdev_write(
//                desc.raw,
//                ch.raw,
//                buf.to_raw(),
//                offset,
//                nbytes,
//                Some(spdk_bdev_io_completion_cb),
//                ptr::null_mut(),
//            );
//        };
//    }

    pub fn spdk_bdev_get_block_size(bdev: SpdkBdev) -> u32 {
        unsafe {
            raw::spdk_bdev_get_block_size(bdev.to_raw())
        }
    }

    pub fn name(&self) -> &str {
        let str_slice: &str;
        unsafe {
            let c_buf = (*self.raw).name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            str_slice = c_str.to_str().unwrap();
        }
        str_slice
    }

    pub fn to_raw(&self) -> *mut raw::spdk_bdev {
        self.raw
    }
}

pub struct SpdkBdevDesc {
    raw: *mut raw::spdk_bdev_desc,
}

impl SpdkBdevDesc {
    pub fn new() -> SpdkBdevDesc {
        SpdkBdevDesc {
            raw: ptr::null_mut(),
        }
    }

    pub fn from_raw(raw: *mut raw::spdk_bdev_desc) -> SpdkBdevDesc {
        unsafe {
            SpdkBdevDesc {
                raw: raw,
            }
        }
    }

    pub fn to_raw(&self) -> *mut raw::spdk_bdev_desc {
        self.raw
    }

    pub fn mut_to_raw(&mut self) -> *mut *mut raw::spdk_bdev_desc {
        &mut self.raw
    }

    pub fn spdk_bdev_desc_get_bdev(&self) -> SpdkBdev {
        let ptr;
        unsafe {
            ptr = raw::spdk_bdev_desc_get_bdev(self.raw);
        }
        SpdkBdev {
            raw: ptr
        }
    }
}

pub struct SpdkIoChannel {
    raw: *mut raw::spdk_io_channel,
}

impl SpdkIoChannel {
    pub fn from_raw(raw: *mut raw::spdk_io_channel) -> SpdkIoChannel {
        unsafe {
            SpdkIoChannel {
                raw: raw,
            }
        }
    }

    pub fn to_raw(&self) -> *mut raw::spdk_io_channel {
        self.raw
    }
}

fn cb_arg<T>(sender: Sender<Result<T, i32>>) -> *mut c_void {
    Box::into_raw(Box::new(sender)) as *const _ as *mut c_void
}

//extern "C" fn spdk_bdev_io_completion_cb(bdev_io: *mut raw::spdk_bdev_io, success: bool, sender_ptr: *mut c_void) {
//    println!("[spdk_bdev_io_completion_cb]");
//    let ret = if !success { Err(-1) } else { Ok(bdev_io) };
//    spdk_app_stop(true);
//}

extern "C" fn spdk_bdev_io_completion_cb(bdev_io: *mut raw::spdk_bdev_io, success: bool, sender_ptr: *mut c_void) {
    println!("[spdk_bdev_io_completion_cb]");
    let sender = unsafe { Box::from_raw(sender_ptr as *mut Sender<Result<*mut raw::spdk_bdev_io, i32>>) };
    let ret = if !success { Err(-1) } else { Ok(bdev_io) };
    sender.send(ret).expect("Receiver is gone");
}
