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
use {raw, AppContext};
use std::ffi::{CString, CStr};
use std::marker;
use std::ptr;

pub struct Bdev<'app_context> {
    raw: *mut raw::spdk_bdev,
    _marker: marker::PhantomData<&'app_context AppContext>,
}

impl<'app_context> Bdev<'app_context> {
    pub fn from_raw(raw: *mut raw::spdk_bdev) -> Bdev<'app_context> {
        unsafe {
            Bdev {
                raw: raw,
                _marker: marker::PhantomData
            }
        }
    }

    pub fn spdk_bdev_first() -> Option<Bdev<'app_context>> {
        unsafe {
            let ptr = raw::spdk_bdev_first();
            if ptr.is_null() {
                None
            } else {
                Some(Bdev::from_raw(ptr))
            }
        }
    }

    pub fn spdk_bdev_next(prev: &Bdev) -> Option<Bdev<'app_context>> {
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

    pub fn to_raw(&self) -> *mut raw::spdk_bdev {
        self.raw
    }
}

pub struct BdevDesc<'app_context> {
    raw: *mut raw::spdk_bdev_desc,
    _marker: marker::PhantomData<&'app_context AppContext>,
}

impl<'app_context> BdevDesc<'app_context> {
    pub fn from_raw(raw: *mut raw::spdk_bdev_desc) -> BdevDesc<'app_context> {
        unsafe {
            BdevDesc {
                raw: raw,
                _marker: marker::PhantomData
            }
        }
    }

    pub fn to_raw(&self) -> *mut raw::spdk_bdev_desc {
        self.raw
    }
}