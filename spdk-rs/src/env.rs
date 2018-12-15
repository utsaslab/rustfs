/*************************************************************************
  > File Name:       env.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    10/10/18
  > Description:
    
    FFI for "env.h"
 ************************************************************************/

use crate::raw;
use std::ffi::{CString, CStr, c_void};
use std::os::raw::{c_char, c_int};
use std::ptr;

pub struct Buf {
    raw : *mut c_void
}

impl Buf {
    pub fn to_raw(&self) -> *mut c_void {
        self.raw
    }
    
    pub fn from_raw(raw: *mut c_void) -> Buf {
        Buf {
            raw: raw,
        }
    }

    /// Fill in the buffer with given content using given fmt
    pub fn fill<S>(&mut self, size: usize, fmt: S, content: S)
    where
        S: Into<String> + Clone,
    {
        let owned_fmt = CString::new(fmt.clone().into()).expect("Couldn't create a string");
        let fmt: *const c_char = owned_fmt.as_ptr();
        let owned_content = CString::new(content.clone().into()).expect("Couldn't create a string");
        let content: *const c_char = owned_content.as_ptr();
        unsafe { raw::snprintf(self.to_raw() as *mut i8, size, fmt, content); }
    }   
}

/// spdk_dma_zmalloc()
pub fn dma_zmalloc(size: usize, align: usize) -> Buf {
    let ptr;
    unsafe {
        ptr = raw::spdk_dma_zmalloc(size, align, ptr::null_mut());
    };
    assert!(!ptr.is_null(), "Failed to malloc");
    Buf {
        raw: ptr
    }
}
