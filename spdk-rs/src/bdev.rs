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
use std::ffi::CStr;
use std::marker;

pub struct Bdev {
    raw: *mut raw::spdk_bdev,
}

//pub struct BdevIter<'bdev_iter> {
//    bdev: &'bdev_iter Bdev,
//}
//
//impl<'bdev_iter> Iterator for BdevIter<'bdev_iter> {
//    type Item = Bdev;
//    fn next(&mut self) -> Option<Bdev> {
//        self.bdev.spdk_bdev_next()
//    }
//}

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

//    pub fn spdk_bdev_next(&self) -> Option<Bdev> {
//        unsafe {
//            let ptr = raw::spdk_bdev_next(self.raw);
//            if ptr.is_null() {
//                None
//            } else {
//                Some(Bdev::from_raw(ptr))
//            }
//        }
//    }

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

//    pub fn spdk_bdev_next(&self) -> Option<Bdev> {
//        unsafe {
//            let ptr = raw::spdk_bdev_next(self.raw);
//            if ptr.is_null() {
//                None
//            } else {
//                Some(Bdev::from_raw(ptr))
//            }
//        }
//    }

    pub fn name(&self) -> &str {
        let str_slice: &str;
        unsafe {
            let c_buf = (*self.raw).name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            str_slice = c_str.to_str().unwrap();
        }
        str_slice
    }

//    pub fn iter(&self) -> BdevIter {
//        BdevIter {
//            bdev: self
//        }
//    }
}