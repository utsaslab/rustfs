/*************************************************************************
  > File Name:       thread.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    12/13/18
  > Description:
    
    FFI for "spdk/thread.h"
 ************************************************************************/

use crate::raw;
use std::ffi::{CString, CStr, c_void};
use std::ptr;

use failure::Error;

#[derive(Debug, Fail)]
pub enum ThreadError {
    #[fail(display = "Failed to allocate thread!")]
    ThreadAllocationError(),
}

pub struct SpdkThread {
    raw: *mut raw::spdk_thread,
}

impl SpdkThread {
    pub fn from_raw(raw: *mut raw::spdk_thread) -> SpdkThread {
        unsafe {
            SpdkThread {
                raw,
            }
        }
    }
}

pub fn allocate_thread<S>(name: S) -> Result<SpdkThread, Error>
where S: Into<String> + Clone,
{
    let name_cstring = CString::new(name.clone().into()).expect("Couldn't create a string");

    let thread_struct = unsafe {
        raw::spdk_allocate_thread(None,
                                  None,
                                  None,
                                  ptr::null_mut(),
                                  name_cstring.as_ptr())
    };
    if thread_struct.is_null() {
        return Err(ThreadError::ThreadAllocationError())?;
    }

    Ok(SpdkThread::from_raw(thread_struct))
}

pub fn free_thread() {
    unsafe {
        raw::spdk_free_thread();
    }
}