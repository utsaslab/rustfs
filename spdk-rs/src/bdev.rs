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
use std::ffi::{CStr};

pub struct Bdev {
    raw : *mut raw::spdk_bdev,
}

impl Default for Bdev{
    fn default() -> Self{
        Self::new()
    }
}

impl Bdev {
    pub fn new() -> Self {
        Bdev{
            raw : ptr::null_mut(),
        }
    }

    pub fn spdk_bdev_first() -> Self{
        let mut bdev = Self::new();
        unsafe {
            bdev.raw = raw::spdk_bdev_first();
        }
        bdev
    }

    pub fn name(&self) -> &str {
        let str_slice: &str;
        unsafe {
            let c_buf = (*self.raw).name;
            let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
            str_slice = c_str.to_str().unwrap();
        }
        str_slice
    }
}