/*************************************************************************
 > File Name:       env.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    10/10/18
 > Description:

   FFI for "env.h"
************************************************************************/

use crate::raw;
use std::ffi::{c_void, CStr, CString};
use std::os::raw::{c_char, c_int};
use std::ptr;

#[derive(Clone)]
pub struct Buf {
    raw: *mut c_void,
}

impl Buf {
    pub fn to_raw(&self) -> *mut c_void {
        self.raw
    }

    pub fn from_raw(raw: *mut c_void) -> Buf {
        Buf { raw: raw }
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
        unsafe {
            raw::snprintf(self.to_raw() as *mut i8, size, fmt, content);
        }
    }

    /// Fill in the buffer with content from "/dev/urandom" with `size`
    pub fn fill_random(&mut self, size: usize) {
        unsafe {
            let owned_path = CString::new("/dev/urandom").unwrap();
            let path: *const c_char = owned_path.as_ptr();
            let fd = libc::open(path, libc::O_RDONLY);
            let ptr = self.to_raw() as *mut c_char;

            let mut left_to_read: isize = size as isize;
            let mut n_to_read: usize = 33554431;
            let mut read_start: usize = 0;

            let mut i: u32 = 0;
            while left_to_read > 0 && i <= 40 {
                if read_start + n_to_read > size {
                    n_to_read = left_to_read as usize;
                }
                debug!("n_to_read: {}", n_to_read);
                let read_size =
                    libc::read(fd, self.to_raw().add(read_start) as *mut c_void, n_to_read);
                debug!("read_size: {}", read_size);
                read_start += read_size as usize;
                debug!("read_start: {}", read_start);
                left_to_read -= read_size as isize;
                debug!("left_to_read: {}", left_to_read);
                i += 1;
            }
            // take a look at what buffer may contain
            for i in 0..30 {
                debug!("{}", *(self.to_raw() as *mut u8).offset(i) as char);
            }
        }
    }

    pub fn read(&self) -> &'static str {
        unsafe { CStr::from_ptr(self.to_raw() as *const i8).to_str().unwrap() }
    }
}

/// spdk_dma_zmalloc()
pub fn dma_zmalloc(size: usize, align: usize) -> Buf {
    let ptr;
    unsafe {
        ptr = raw::spdk_dma_zmalloc(size, align, ptr::null_mut());
    };
    assert!(!ptr.is_null(), "Failed to malloc");
    Buf { raw: ptr }
}
