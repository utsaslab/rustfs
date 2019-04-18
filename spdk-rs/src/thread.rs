/*************************************************************************
 > File Name:       thread.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    12/13/18
 > Description:

   FFI for "spdk/thread.h"
************************************************************************/

use crate::raw;
use std::ffi::CString;
use std::ptr;

use failure::Error;

#[derive(Debug, Fail)]
pub enum ThreadError {
    #[fail(display = "Failed to allocate thread!")]
    ThreadAllocationError(),
}

#[derive(Clone)]
pub struct SpdkIoChannel {
    raw: *mut raw::spdk_io_channel,
}

impl SpdkIoChannel {
    pub fn from_raw(raw: *mut raw::spdk_io_channel) -> SpdkIoChannel {
        SpdkIoChannel { raw: raw }
    }

    pub fn to_raw(&self) -> *mut raw::spdk_io_channel {
        self.raw
    }
}

#[allow(dead_code)]
pub struct SpdkThread {
    raw: *mut raw::spdk_thread,
}

impl SpdkThread {
    pub fn from_raw(raw: *mut raw::spdk_thread) -> SpdkThread {
        SpdkThread { raw }
    }
}

pub fn allocate_thread<S>(name: S) -> Result<SpdkThread, Error>
where
    S: Into<String> + Clone,
{
    let name_cstring = CString::new(name.clone().into()).expect("Couldn't create a string");

    let thread_struct = unsafe {
        raw::spdk_allocate_thread(None, None, None, ptr::null_mut(), name_cstring.as_ptr())
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

pub fn put_io_channel(channel: SpdkIoChannel) {
    unsafe { raw::spdk_put_io_channel(channel.to_raw()) }
}
