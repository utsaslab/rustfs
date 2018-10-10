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

use raw;
use Bdev;
use BdevDesc;

use std::ffi::{CString, CStr};
use std::os::raw::{c_void, c_char, c_int};
use std::ptr;


pub struct AppContext {
    bdev: *mut raw::spdk_bdev,
    bdev_desc: *mut raw::spdk_bdev_desc,
    bdev_io_channel: *mut raw::spdk_io_channel,
    buff: *mut c_char,
    bdev_name: *const c_char,
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

    pub fn bdev(&self) -> Option<Bdev> {
        Some(Bdev::from_raw(self.bdev))
    }

    /// set bdev field based on the bdev_name
    ///
    /// **NOTE:** The implementation can be improved becaseu we essentially
    /// duplicate code of bdev_name. The reason we doing so as a way to workaround
    /// the borrow checker. See more info about the issue I'm facing during the implementation
    /// [here](https://stackoverflow.com/questions/52709147/how-to-workaround-the-coexistence-of-a-mutable-and-immutable-borrow)
    pub fn set_bdev(&mut self) -> Result<i32, String> {
        let str_slice;
        unsafe {
            let c_buf: *const c_char = self.bdev_name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            str_slice = c_str.to_str().unwrap();
        }
        let bdev = Bdev::spdk_bdev_get_by_name(str_slice);
        match bdev {
            Err(E) => {
                let s = E.to_owned();
                Err(s)
            }
            Ok(T) => {
                self.bdev = T.to_raw();
                Ok(0)
            }
        }
    }

    pub fn bdev_desc(&self) -> Option<BdevDesc> {
        Some(BdevDesc::from_raw(self.bdev_desc))
    }

    /// # Parameters
    ///
    /// - self: the context when start the SPDK framework
    /// - write: true is read/write access requested, false if read-only
    ///
    /// Note: Below shows an native implementation of the spdk_bdev_open(), which might have less overhead.
    /// Right now, we invoke the spdk_bdev_open() under bdev.rs to reach the desired goal. We have a one more
    /// layer indirection. Doing so makes sense to the SPDK API: we want to have spdk_bdev_open() method
    /// under bdev.rs instead of the context.rs. However, the overhead comes from we have to pack the self.bdev_desc
    /// first to be modified and send to the bdev.rs spdk_bdev_open() and unpack the object to update the field
    /// within the context struct.
    ///
    /// We only modify the spdk_bdev_open() to follow this philosophy of idea. Other original SPDK functions
    /// that implemented under context.rs instead of bdev.rs may apply as well. We don't change them all of them for now
    /// because there are trade-off between two ways of implementation and we may adopt one whenever we need.
    ///
    /// ```rust
    ///   pub fn spdk_bdev_open(&mut self, write: bool) -> Result<i32, String> {
    ///        unsafe {
    ///            let rc = raw::spdk_bdev_open(self.bdev, write, None, ptr::null_mut(), &mut self.bdev_desc);
    ///            match rc != 0 {
    ///                true => {
    ///                    let s = format!("Could not open bdev: {}", self.bdev_name());
    ///                    Err(s)
    ///                }
    ///                false => {
    ///                    Ok(0)
    ///                }
    ///            }
    ///        }
    ///    }
    /// ```
    pub fn spdk_bdev_open(&mut self, write: bool) -> Result<i32, String> {
        let mut bdev_desc = BdevDesc::from_raw(self.bdev_desc);
        match Bdev::spdk_bdev_open(&Bdev::from_raw(self.bdev), write, &mut bdev_desc) {
            Err(_e) => {
                let s = format!("Could not open bdev: {}", self.bdev_name());
                Err(s)
            }
            Ok(_) => {
                self.bdev_desc = bdev_desc.to_raw();
                Ok(0)
            }
        }
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
        unsafe {
            raw::spdk_put_io_channel(self.bdev_io_channel)
        }
    }

    pub fn allocate_buff(&mut self) -> Result<i32, String> {
        unsafe {
            self.buff = raw::spdk_dma_zmalloc(raw::spdk_bdev_get_block_size(self.bdev) as usize,
                                              raw::spdk_bdev_get_buf_align(self.bdev),
                                              ptr::null_mut()) as *mut c_char;
            match self.buff.is_null() {
                true => {
                    let s = format!("Failed to allocate buffer");
                    Err(s)
                }
                false => {
                    Ok(0)
                }
            }
        }
    }
}