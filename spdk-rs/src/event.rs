/*************************************************************************
  > File Name:       event.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    10/2/18
  > Description:
    
    FFI for "event.h"
 ************************************************************************/

use {raw, Bdev, BdevDesc};
use failure::Error;
use std::os::raw::{c_char, c_void};
use std::ffi::{CStr, CString};
use std::ptr;

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

    pub fn set_bdev_name(&mut self, name: &str) {
        self.bdev_name = CString::new(name)
            .expect("Couldn't create a string")
            .into_raw()
    }

    pub fn bdev_name(&self) -> &str {
        unsafe {
            let c_buf: *const c_char = self.bdev_name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            let str_slice: &str = c_str.to_str().unwrap();
            str_slice
        }
    }

    pub fn bdev(&self) -> Option<Bdev> {
        Some(Bdev::from_raw(self.bdev))
    }

    /// set bdev field based on the bdev_name
    ///
    /// **NOTE:** The implementation can be improved becaseu we essentially
    /// duplicate code of bdev_name. The reason we doing so as a way to workaround
    /// the borrow checker. See more info about the issue I'm facing during the implementation
    /// [here](https://stackoverflow.com/questions/52709147/how-to-workaround-the-coexistence-of-a-mutable-and-immutable-borrow)
    pub fn set_bdev(&mut self) -> Result<i32, String> {
        let str_slice;
        unsafe {
            let c_buf: *const c_char = self.bdev_name;
            let c_str: &CStr = CStr::from_ptr(c_buf);
            str_slice = c_str.to_str().unwrap();
        }
        let bdev = Bdev::spdk_bdev_get_by_name(str_slice);
        match bdev {
            Err(E) => {
                let s = E.to_owned();
                Err(s)
            }
            Ok(T) => {
                self.bdev = T.to_raw();
                Ok(0)
            }
        }
    }

    pub fn bdev_desc(&self) -> Option<BdevDesc> {
        Some(BdevDesc::from_raw(self.bdev_desc))
    }

    /// # Parameters
    ///
    /// - context: the context when start the SPDK framework
    /// - write: true is read/write access requested, false if read-only
    ///
    pub fn spdk_bdev_open(&mut self, write: bool) -> Result<i32, String> {
        unsafe {
            let rc = raw::spdk_bdev_open(self.bdev, write, None, ptr::null_mut(), &mut self.bdev_desc);
            match rc != 0 {
                true => {
                    let s = format!("Could not open bdev: {}", self.bdev_name());
                    Err(s)
                }
                false => {
                    Ok(0)
                }
            }
        }
    }

    pub fn spdk_bdev_close(&mut self) {
        unsafe {
            raw::spdk_bdev_close(self.bdev_desc);
        }
    }

    pub fn spdk_bdev_get_io_channel(&mut self) -> Result<i32, String> {
        unsafe {
            let ptr = raw::spdk_bdev_get_io_channel(self.bdev_desc);
            match ptr.is_null() {
                true => {
                    let s = format!("Could not create bdev I/O channel!!");
                    Err(s)
                }
                false => {
                    self.bdev_io_channel = ptr;
                    Ok(0)
                }
            }
        }
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

    pub fn start<F>(mut self, f: F) -> Result<(), Error>
        where
            F: FnMut() -> (),
    {
        let user_data = &f as *const _ as *mut c_void;

        extern "C" fn start_wrapper<F>(closure: *mut c_void, _: *mut c_void)
            where
                F: FnMut() -> (),
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
//            if (context.buff != ptr::null_mut()) {
//                raw::spdk_dma_free(context.buff as *mut c_void);
//            }
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