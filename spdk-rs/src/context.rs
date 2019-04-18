/*************************************************************************
 > File Name:       context.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    10/10/18
 > Description:

   An abstraction of the context that is needed for the SPDK framework.
   This file is not part of the original spdk C API. I implement this
   because I forsee any SPDK-based application may need to define the context struct.

************************************************************************/

use crate::bdev;
use crate::raw;
use crate::thread;
use crate::Buf;
use crate::SpdkBdev;
use crate::SpdkBdevDesc;
use crate::SpdkBdevIO;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::ptr;

use failure::Error;

pub trait SpdkBdevIoCompletionCb {
    fn callback(&mut self, bdev_io: SpdkBdevIO, success: bool);
}

pub struct AppContext {
    bdev: *mut raw::spdk_bdev,
    bdev_desc: *mut raw::spdk_bdev_desc,
    bdev_io_channel: *mut raw::spdk_io_channel,
    buff: *mut c_char,
    bdev_name: *const c_char,
}

impl Clone for AppContext {
    fn clone(&self) -> AppContext {
        AppContext {
            bdev: self.bdev,
            bdev_desc: self.bdev_desc,
            bdev_io_channel: self.bdev_io_channel,
            buff: self.buff,
            bdev_name: self.bdev_name,
        }
    }
}

impl AppContext {
    pub fn new() -> AppContext {
        AppContext {
            bdev: ptr::null_mut(),
            bdev_desc: ptr::null_mut(),
            bdev_io_channel: ptr::null_mut(),
            buff: ptr::null_mut(),
            bdev_name: ptr::null_mut(),
        }
    }

    pub fn set_bdev_name(&mut self, name: &str) {
        self.bdev_name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn bdev_name(&self) -> &str {
        unsafe {
            let c_buf: *const c_char = self.bdev_name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice
        }
    }

    pub fn bdev(&self) -> Option<SpdkBdev> {
        Some(SpdkBdev::from_raw(self.bdev))
    }

    /// set bdev field based on the bdev_name
    ///
    /// **NOTE:** The implementation can be improved becaseu we essentially
    /// duplicate code of bdev_name. The reason we doing so as a way to workaround
    /// the borrow checker. See more info about the issue I'm facing during the implementation
    /// [here](https://stackoverflow.com/questions/52709147/how-to-workaround-the-coexistence-of-a-mutable-and-immutable-borrow)
    pub fn set_bdev(&mut self) -> Result<(), Error> {
        let str_slice;
        unsafe {
            let c_buf: *const c_char = self.bdev_name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            str_slice = c_str.to_str().unwrap();
        }
        let bdev = bdev::get_by_name(str_slice);
        match bdev {
            Err(e) => Err(e)?,
            Ok(t) => {
                self.bdev = t.to_raw();
                Ok(())
            }
        }
    }

    pub fn bdev_desc(&self) -> Option<SpdkBdevDesc> {
        Some(SpdkBdevDesc::from_raw(self.bdev_desc))
    }

    pub fn spdk_bdev_close(&mut self) {
        unsafe {
            raw::spdk_bdev_close(self.bdev_desc);
        }
    }

    pub fn spdk_bdev_get_io_channel(&mut self) -> Result<i32, String> {
        unsafe {
            let ptr = raw::spdk_bdev_get_io_channel(self.bdev_desc);
            match ptr.is_null() {
                true => {
                    let s = format!("Could not create bdev I/O channel!!");
                    Err(s)
                }
                false => {
                    self.bdev_io_channel = ptr;
                    Ok(0)
                }
            }
        }
    }

    pub fn spdk_bdev_put_io_channel(&self) {
        unsafe { raw::spdk_put_io_channel(self.bdev_io_channel) }
    }

    #[allow(unused_variables)]
    unsafe extern "C" fn spdk_bdev_io_completion_cb<F>(
        bdev_io: *mut raw::spdk_bdev_io,
        success: bool,
        cb_arg: *mut c_void,
    ) where
        F: FnMut() -> (),
    {
        let opt_closure = cb_arg as *mut F;
        (*opt_closure)()
    }

    pub fn spdk_bdev_write<F>(&self, offset: u64, cb: F) -> Result<i32, String>
    where
        F: FnMut() -> (),
    {
        let callback = Box::new(cb);
        let ret = unsafe {
            raw::spdk_bdev_write(
                self.bdev_desc,
                self.bdev_io_channel,
                self.buff as *mut c_void,
                offset,
                raw::spdk_bdev_get_block_size(self.bdev) as u64,
                Some(AppContext::spdk_bdev_io_completion_cb::<F>),
                &*callback as *const _ as *mut c_void,
            )
        };
        std::mem::forget(callback);
        match ret == 0 {
            true => Ok(0),
            false => Result::Err(format!("Could not write to the device")),
        }
    }

    pub fn allocate_buff(&mut self) -> Result<i32, String> {
        unsafe {
            self.buff = raw::spdk_dma_zmalloc(
                raw::spdk_bdev_get_block_size(self.bdev) as usize,
                raw::spdk_bdev_get_buf_align(self.bdev),
                ptr::null_mut(),
            ) as *mut c_char;
            match self.buff.is_null() {
                true => {
                    let s = format!("Failed to allocate buffer");
                    Err(s)
                }
                false => Ok(0),
            }
        }
    }

    pub fn bdev_io_channel(&self) -> thread::SpdkIoChannel {
        thread::SpdkIoChannel::from_raw(self.bdev_io_channel)
    }

    pub fn buff(&self) -> Buf {
        Buf::from_raw(self.buff as *mut c_void)
    }

    /// hello_nvme_bdev specific function:
    /// write message string into the the allocated buff
    pub fn write_buff(&mut self, message: &str) {
        let owned_fmt = CString::new("%s\n").unwrap();
        let fmt: *const c_char = owned_fmt.as_ptr();
        let owned_content = CString::new(message).unwrap();
        let content: *const c_char = owned_content.as_ptr();
        unsafe {
            raw::snprintf(
                self.buff,
                raw::spdk_bdev_get_block_size(self.bdev) as usize,
                fmt,
                content,
            );
        }
    }
}

//impl SpdkBdevIoCompletionCb for &mut AppContext{
//    fn callback(&mut self, bdev_io: SpdkBdevIO, success: bool) {
//        spdk_app_stop(true);
//    }
//}
