/*************************************************************************
  > File Name:       bdev_module.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    10/16/18
  > Description:
    
    FFI for "bdev_module.h"
 ************************************************************************/

use crate::raw;
use std::ptr;

pub struct SpdkBdevIO {
    raw : *mut raw::spdk_bdev_io
}

impl SpdkBdevIO {
    pub fn from_raw(raw: *mut raw::spdk_bdev_io) -> SpdkBdevIO {
        unsafe {
            SpdkBdevIO {
                raw: raw,
            }
        }
    }

    pub fn to_raw(&self) -> *mut raw::spdk_bdev_io {
        self.raw
    }

    pub fn new() -> Self{
        SpdkBdevIO {
            raw: ptr::null_mut()
        }
    }
}