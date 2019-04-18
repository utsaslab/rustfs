/*************************************************************************
 > File Name:       env.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    10/10/18
 > Description:

   FFI for "env.h". This file also contains some helper functions that work with
   the Buf.

************************************************************************/

use crate::raw;
use std::ffi::{c_void, CStr, CString};
use std::fs;

use std::os::raw::c_char;
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

            let mut left_to_read: isize = size as isize;
            let mut n_to_read: usize = 33554431;
            let mut read_start: usize = 0;

            while left_to_read > 0 {
                if read_start + n_to_read > size {
                    n_to_read = left_to_read as usize;
                }
                debug!("n_to_read: {}", n_to_read);
                let mut read_size = libc::read(
                    fd,
                    self.to_raw().add(read_start) as *mut libc::c_void,
                    n_to_read,
                );
                if read_size == -1 {
                    // there is error, we try to fill the fixed content instead
                    read_size = self.fill_fixed(size, "AAA") as isize;
                }
                debug!("read_size: {}", read_size);
                read_start += read_size as usize;
                debug!("read_start: {}", read_start);
                left_to_read -= read_size as isize;
                debug!("left_to_read: {}", left_to_read);
            }
        }
    }

    /// Fill in the buffer with fixed content specified by `pattern`
    pub fn fill_fixed(&mut self, size: usize, pattern: &str) -> usize {
        unsafe {
            let mut left_to_write: isize = size as isize;
            let mut write_start: usize = 0;
            let mut n_to_write = pattern.len();
            let mut tot_write = 0;

            // convert pattern from &str to *mut c_char
            let owned_content = CString::new(pattern).unwrap();
            let content: *const c_char = owned_content.as_ptr();

            let owned_fmt = CString::new("%s").unwrap();
            let fmt: *const c_char = owned_fmt.as_ptr();

            while left_to_write > 0 {
                if write_start + n_to_write > size {
                    n_to_write = left_to_write as usize;
                }
                let write_size = libc::snprintf(
                    self.to_raw().add(write_start) as *mut c_char,
                    n_to_write + 1,
                    fmt,
                    content,
                );
                assert!(write_size == pattern.len() as i32);
                tot_write += write_size;
                write_start += write_size as usize;
                left_to_write -= write_size as isize;
            }
            tot_write as usize
        }
    }

    /// Fill in the buffer with content from file `filename` with start position `start_pos` and
    /// with size `usize` (in bytes)
    pub fn fill_from_file(
        &mut self,
        filename: &str,
        start_pos: usize,
        size: usize,
    ) -> libc::ssize_t {
        match fs::metadata(filename) {
            Ok(_) => {}
            Err(_) => panic!("{} does not exist", filename),
        };
        unsafe {
            let owned_filename = CString::new(filename).unwrap();
            let filename_ptr: *const c_char = owned_filename.as_ptr();

            let owned_mode = CString::new("r").unwrap();
            let mode_ptr: *const c_char = owned_mode.as_ptr();

            let fp: *mut libc::FILE = libc::fopen(filename_ptr, mode_ptr);
            libc::fseek(fp, start_pos as i64, libc::SEEK_SET);
            let fd: libc::c_int = libc::fileno(fp);
            let num_read = libc::read(fd, self.to_raw() as *mut libc::c_void, size);
            // add null terminator to the end of string
            //*(self.to_raw() as *mut u8).offset(num_read + 1 as isize) = '\0' as u8;
            if num_read == -1 {
                libc::fclose(fp);
                println!("read fails");
                libc::exit(1);
            }
            num_read
        }
    }

    pub fn read(&self) -> &'static str {
        unsafe {
            CStr::from_ptr(self.to_raw() as *const c_char)
                .to_str()
                .unwrap()
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
    Buf { raw: ptr }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_fill_fixed() {
        unsafe {
            let mut buffer_size = 5;
            let mut buffer = libc::malloc(mem::size_of::<c_char>() * buffer_size);
            let mut buf = Buf::from_raw(buffer as *mut c_void);
            let write_size = buf.fill_fixed(buffer_size, "A");
            assert!(write_size == buffer_size);
            for i in 0..buffer_size {
                assert!(*(buf.to_raw() as *mut u8).offset(i as isize) as char == 'A');
            }
            libc::free(buffer);
        }
    }

    #[test]
    fn test_fill_from_file() -> std::io::Result<()> {
        let filename = "/tmp/test_fill_from_file.txt";
        let outname = "/tmp/out.txt";
        let test_string: String = String::from("ABCDEFGHIJKLMN");
        let length = test_string.len();
        assert!(length == 14);
        let mut output = fs::File::create(filename)?;
        write!(output, "{}", test_string)?;
        let buffer_size = 3;
        unsafe {
            for i in 0..length {
                let mut buffer = libc::malloc(mem::size_of::<c_char>() * buffer_size);
                let mut buf = Buf::from_raw(buffer as *mut c_void);
                let num_read = buf.fill_from_file(filename, i * buffer_size, buffer_size);
                for j in 0..num_read as usize {
                    assert!(
                        *(buf.to_raw() as *mut u8).offset(j as isize) as char
                            == test_string.chars().nth(i * buffer_size + j).unwrap()
                    );
                }
                libc::free(buffer);
            }
        }
        Ok(())
    }
}
