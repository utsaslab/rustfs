/*************************************************************************
  > File Name:       bdev.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    9/16/18
  > Description:
    
    FFI for "bdev.h"
 ************************************************************************/

use raw;
use std::ptr;
use std::ffi::{CString, CStr};

pub struct Bdev {
    raw: *mut raw::spdk_bdev,
}

impl Bdev {
    unsafe fn from_raw(raw: *mut raw::spdk_bdev) -> Bdev {
        Bdev {
            raw: raw,
        }
    }

    pub fn spdk_bdev_first() -> Option<Bdev> {
        unsafe {
            let ptr = raw::spdk_bdev_first();
            if ptr.is_null() {
                None
            } else {
                Some(Bdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_next(prev: &Bdev) -> Option<Bdev> {
        unsafe {
            let ptr = raw::spdk_bdev_next(prev.raw);
            if ptr.is_null() {
                None
            } else {
                Some(Bdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_get_by_name(bdev_name: &str) -> Result<Bdev, String> {
        unsafe {
            let c_str = CString::new(bdev_name).unwrap();
            let c_str_ptr = c_str.as_ptr();
            let ptr = raw::spdk_bdev_get_by_name(c_str_ptr);
            if ptr.is_null() {
                Result::Err(format!("Could not find the bdev: {}", bdev_name))
            }
            else {
                Ok(Bdev::from_raw(ptr))
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
}