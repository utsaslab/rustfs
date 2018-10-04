/*************************************************************************
  > File Name:       event.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    10/2/18
  > Description:
    
    FFI for "event.h"
 ************************************************************************/

use raw;
use failure::Error;
use std::os::raw::{c_char, c_void};
use std::ffi::CString;
use std::ptr;
use std::mem;

#[derive(Debug, Fail)]
enum AppError {
    #[fail(display = "Spdk failed to start: {}", _0)]
    StartupError(i32),
}

pub struct AppContext {
    bdev: *mut raw::spdk_bdev,
    bdev_desc: *mut raw::spdk_bdev_desc,
    bdev_io_channel: *mut raw::spdk_io_channel,
    buff: *mut c_char,
    bdev_name: *const c_char,
}

impl AppContext {
    pub fn new() -> AppContext {
        AppContext {
            bdev: ptr::null_mut(),
            bdev_desc: ptr::null_mut(),
            bdev_io_channel: ptr::null_mut(),
            buff: ptr::null_mut(),
            bdev_name: ptr::null_mut(),
        }
    }

    pub fn bdev_name(&mut self, name: &str) {
        self.bdev_name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }
}


// tuple struct: https://doc.rust-lang.org/1.9.0/book/structs.html
// https://stackoverflow.com/questions/30339831/what-are-some-use-cases-for-tuple-structs
#[derive(Default)]
pub struct AppOpts(raw::spdk_app_opts);

impl AppOpts {
    pub fn new() -> Self {
        let mut opts: raw::spdk_app_opts = Default::default();
        unsafe {
            raw::spdk_app_opts_init(&mut opts as *mut raw::spdk_app_opts);
        }
        AppOpts(opts)
    }

    pub fn name(&mut self, name: &str) {
        self.0.name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn config_file(&mut self, config_file: &str) {
        self.0.config_file = CString::new(config_file)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn start<F>(mut self, f: F, context: AppContext) -> Result<(), Error>
        where
            F: Fn() -> (),
    {
        let user_data = &f as *const _ as *mut c_void;

        extern "C" fn start_wrapper<F>(closure: *mut c_void, _: *mut c_void)
            where
                F: Fn() -> (),
        {
            let opt_closure = closure as *mut F;
            unsafe { (*opt_closure)() }
        }

        let ret = unsafe {
            let self_ref = &mut self;
            let opts_ref = &mut self_ref.0;
            raw::spdk_app_start(
                opts_ref as *mut raw::spdk_app_opts,
                Some(start_wrapper::<F>),
                user_data,
                ptr::null_mut(),
            )
        };

        unsafe {
            if (context.buff != ptr::null_mut()) {
                raw::spdk_dma_free(context.buff as *mut c_void);
            }
            raw::spdk_app_fini();
        }

        if ret == 0 {
            Ok(())
        } else {
            Err(AppError::StartupError(ret))?
        }
    }
}

pub fn app_stop(success: bool) {
    unsafe {
        raw::spdk_app_stop(if success { 0 } else { -1 });
    };
}

impl Drop for AppOpts {
    fn drop(&mut self) {
        drop_if_not_null(self.0.name as *mut c_char);
        drop_if_not_null(self.0.config_file as *mut c_char);
    }
}

fn drop_if_not_null(string: *mut c_char) {
    if !string.is_null() {
        unsafe { CString::from_raw(string as *mut c_char) };
    }
}