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
