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
use {raw, AppContext, Buf, SpdkBdevIO};
use std::ffi::{CString, CStr, c_void};
use std::marker;
use std::ptr;

pub struct SpdkBdev {
    raw: *mut raw::spdk_bdev,
}

impl SpdkBdev {
    pub fn from_raw(raw: *mut raw::spdk_bdev) -> SpdkBdev {
        unsafe {
            SpdkBdev {
                raw: raw,
            }
        }
    }

    pub fn spdk_bdev_first() -> Option<SpdkBdev> {
        unsafe {
            let ptr = raw::spdk_bdev_first();
            if ptr.is_null() {
                None
            } else {
                Some(SpdkBdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_next(prev: &SpdkBdev) -> Option<SpdkBdev> {
        unsafe {
            let ptr = raw::spdk_bdev_next(prev.raw);
            if ptr.is_null() {
                None
            } else {
                Some(SpdkBdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_open(bdev: &SpdkBdev, write: bool, bdev_desc: &mut SpdkBdevDesc) -> Result<i32, String> {
        unsafe {
            let rc = raw::spdk_bdev_open(bdev.to_raw(), write, None, ptr::null_mut(), bdev_desc.mut_to_raw());
            match rc != 0 {
                true => {
                    let s = format!("Could not open bdev: {}", bdev.name());
                    Err(s)
                }
                false => {
                    Ok(0)
                }
            }
        }
    }

    pub fn spdk_bdev_get_by_name(bdev_name: &str) -> Result<SpdkBdev, String> {
        unsafe {
            let c_str = CString::new(bdev_name).unwrap();
            let c_str_ptr = c_str.as_ptr();
            let ptr = raw::spdk_bdev_get_by_name(c_str_ptr);
            if ptr.is_null() {
                Result::Err(format!("Could not find the bdev: {}", bdev_name))
            } else {
                Ok(SpdkBdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_write<F>(desc: SpdkBdevDesc,
                              ch: SpdkIoChannel,
                              buf: Buf,
                              offset: u64,
                              nbytes: u64,
                              f: F) -> Result<i32, String>
        where
            F: FnMut(*mut raw::spdk_bdev_io, bool) -> (), {
        let user_data = &f as *const _ as *mut c_void;

        extern "C" fn start_wrapper<F>(bdev_io: *mut raw::spdk_bdev_io, success: bool, closure: *mut c_void)
            where
                F: FnMut(*mut raw::spdk_bdev_io, bool) -> (),
        {
            let opt_closure = closure as *mut F;
            unsafe { (*opt_closure)(bdev_io, success) }
        }

        let ret;
        unsafe {
            ret = raw::spdk_bdev_write(
                desc.to_raw(),
                ch.to_raw(),
                buf.to_raw(),
                offset,
                nbytes,
                Some(start_wrapper::<F>),
                user_data,
            );
        };
        match ret != 0 {
            true => {
                let s = format!("Error while writing to bdev");
                Err(s)
            }
            false => {
                Ok(0)
            }
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