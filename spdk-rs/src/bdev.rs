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
    bdev_ptr : *mut raw::spdk_bdev,
}

impl Default for Bdev{
    fn default() -> Self{
        Self::new()
    }
}

impl Bdev {
    pub fn new() -> Self {
        Bdev{
            bdev_ptr : ptr::null_mut(),
        }
    }

    pub fn spdk_bdev_first() -> Self{
        let mut bdev = Self::new();
        unsafe {
            bdev.bdev_ptr = raw::spdk_bdev_first();
        }
        bdev
    }

    pub fn name(&self) -> &str {
        let str_slice: &str;
        unsafe {
            let c_buf = (*self.bdev_ptr).name;
            let c_str: &CStr = unsafe { CStr::from_ptr(c_buf) };
            str_slice = c_str.to_str().unwrap();
        }
        str_slice
    }
}