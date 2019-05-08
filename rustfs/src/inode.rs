/*************************************************************************
  > File Name:       inode.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    9/21/18
  > Description:

  This file contains the implementation of the inode.
 ************************************************************************/

extern crate spdk_rs;
#[macro_use]
extern crate arrayref;

use time;
use time::Timespec;
use std::mem;
use std::ptr;
use std::ptr::copy_nonoverlapping;

const BLOCK_SIZE: usize = 512;
const LIST_SIZE: usize = 16;

type Page = Box<([u8; BLOCK_SIZE])>;
type Entry = Page;
type EntryList = TList<Entry>; // TODO: Option<TList> for lazy loading
type DoubleEntryList = TList<EntryList>;
pub type TList<T> = Box<([Option<T>; LIST_SIZE])>;

#[inline(always)]
fn ceil_div(x: usize, y: usize) -> usize {
    return (x + y - 1) / y;
}

#[inline(always)]
pub fn create_tlist<T>() -> TList<T> {
    let mut list: TList<T> = Box::new(unsafe { mem::uninitialized() });
    for x in list.iter_mut() { unsafe { ptr::write(x, None); } };
    list
}

pub struct Inode {
    fs: &FS,
    inum: usize,

    dirtype: usize,
    single: Option<usize>,
    double: Option<usize>,
    size: usize,
}

impl Inode {
    pub fn new(fs: &FS, dirtype: usize, inum: usize) -> Inode {
        Inode {
            fs: fs,
            inum: inum,
            dirtype: dirtype,
            single: None,
            double: None,
            size: 0,
        }
    }

    fn read_inode(&self){
        let offset = &self.fs.inode_base + &self.inum * Inode::size();
        let blk = offset / BLOCK_SIZE;
        let blk_offset = offset % BLOCK_SIZE;
        let mut read_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
        &self.fs.device.read(read_buf, blk, BLOCK_SIZE);
        let mut buf = read_buf.read_bytes(BLOCK_SIZE);
        let mut content = &buf[blk_offset..blk_offset + Inode::size()];
        unsafe{
            self.dirtype = mem::transmute::<[u8;8], usize>(*array_ref![content, 0, 8]);
            self.size = mem::transmute::<[u8;8], usize>(*array_ref![content, 8, 8]);
            self.single = Some(mem::transmute::<[u8;8], usize>(*array_ref![content, 16, 8]));
            self.double = Some(mem::transmute::<[u8;8], usize>(*array_ref![content, 24, 8]));
        }
    }

    fn parse_entry(raw_read: &[u8], index: usize) -> usize {
        let start = index * 8;
        let content = &raw_read[start..start + 8];
        let entry: usize;
        unsafe{
            entry = mem::transmute::<[u8; 8], usize>(*array_ref![content, 0, 8]); 
        }
        entry
    }

    fn write_inode(&self){
        // TODO: add unit test
        let offset = &self.fs.inode_base + &self.inum * Inode::size();
        let blk = offset / BLOCK_SIZE;
        let blk_offset = offset % BLOCK_SIZE;
        let mut read_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
        &self.fs.device.read(read_buf, blk, BLOCK_SIZE);
        let mut buf = read_buf.read_bytes(BLOCK_SIZE);
        let mut content = &buf[blk_offset..blk_offset + Inode::size()];
        unsafe{
            let tmp = mem::transmute::<usize, [u8;8]>(self.dirtype);
            content[0..8].copy_from_slice(&tmp[0..8]);
            let tmp = mem::transmute::<usize, [u8;8]>(self.size);
            content[8..16].copy_from_slice(&tmp[0..8]);
            let tmp = mem::transmute::<usize, [u8;8]>(self.single.unwrap());
            content[16..24].copy_from_slice(&tmp[0..8]);
            let tmp = mem::transmute::<usize, [u8;8]>(self.double.unwrap());
            content[24..32].copy_from_slice(&tmp[0..8]);
        }
        let mut write_buf = read_buf;
        write_buf.fill_bytes(buf);
        &self.fs.device.write(write_buf, blk, BLOCK_SIZE);
    }

    pub fn size() -> usize{
        32
    }

    // read inode metadata and return block number
    fn get_or_alloc_page<'a>(&'a mut self, num: usize) -> usize {
        if num >= LIST_SIZE + 1 {
            panic!("Maximum file size exceeded!")
        };

        let need_update = false;
        &mut self.read_inode();

        // Getting a pointer to the page
        let page = if num == 0 {
            if &self.single.is_none() {
                //                if &self.size == 0 {
                single = &mut self.fs.alloc_block();
                need_update = true;
                //                }else{
                //                    &mut self.read_inode();
                //                }
            }
            single
        } else {
            // if the page num is in the doubly-indirect list. We allocate a new
            // entry list where necessary (*entry_list = ...)
            let index = num - 1;
            if &self.double.is_none() {
                //                if &self.size <= BLOCK_SIZE {
                double = &mut self.fs.alloc_block();
                need_update = true;
                //                }else{
                //                }
            }
            let mut read_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
            let offset = &self.fs.data_base + &self.double.unwrap() * BLOCK_SIZE;
            &mut self.fs.device.read(read_buf, offset, BLOCK_SIZE);
            let entry = Inode::parse_entry(&read_buf.read_bytes(), index);

            entry
        };

        if need_update{
            &self.write_inode();
        }
        page
    }

    fn get_page<'a>(&'a self, num: usize) -> usize {
        if num * BLOCK_SIZE >= &self.size {
            panic!("Page does not exist.")
        };
        &mut self.read_inode();

        if num == 0 {
            0
        } else {
            let index = num - 1;
            let mut read_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
            let offset = &self.fs.data_base + &self.double.unwrap() * BLOCK_SIZE;
            &mut self.fs.device.read(read_buf, offset, BLOCK_SIZE);
            let entry = Inode::parse_entry(&read_buf.read_bytes(), index);
            entry
        }
    }

    pub fn write(&mut self, offset: usize, data: &[u8]) -> usize {
        let mut written = 0;
        let mut block_offset = offset % BLOCK_SIZE; // offset from first block

        let start = offset / BLOCK_SIZE; // first block to act on
        let blocks_to_act_on = ceil_div(block_offset + data.len(), BLOCK_SIZE);

        for i in 0..blocks_to_act_on {
            // Resetting the block offset after first pass since we want to read from
            // the beginning of the block after the first time.
            if block_offset != 0 && i > 0 { block_offset = 0 };

            // Need to account for offsets from first and last blocks
            let num_bytes = if i == blocks_to_act_on - 1 {
                data.len() - written
            } else {
                BLOCK_SIZE - block_offset
            };

            // Finding our block, writing to it
            let page = self.get_or_alloc_page(start + i);

            // TODO: check this!
            let pg_offset = self.fs.data_base + page * BLOCK_SIZE;
            let mut read_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
            &self.fs.device.read(&mut read_buf, pg_offset, BLOCK_SIZE);
            let disk_page = read_buf.read_bytes();
            // let slice = array_mut_ref![disk_page, block_offset, num_bytes]; 
            let slice = &mut disk_page[block_offset..(block_offset + num_bytes)];
            // written += slice.copy_from(data.slice(written, written + num_bytes));
            unsafe {
                // TODO: This may be extremely slow! Use copy_nonoverlapping, perhaps.
                let src = data[written..(written + num_bytes)].as_ptr();
                copy_nonoverlapping(src, slice.as_mut_ptr(), num_bytes);
            }
            let mut write_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
            write_buf.fill_bytes(disk_page);
            &self.fs.device.write(&mut write_buf, offset, BLOCK_SIZE);

            written += num_bytes;
        }

        let last_byte = offset + written;
        if self.size < last_byte { self.size = last_byte; }

        //        let time_now = time::get_time();
        //        self.mod_time = time_now;
        //        self.access_time = time_now;

        written
    }

    pub fn read(&self, offset: usize, data: &mut [u8]) -> usize {
        let mut read = 0;
        let mut block_offset = offset % BLOCK_SIZE; // offset from first block
        let start = offset / BLOCK_SIZE; // first block to act on
        let blocks_to_act_on = ceil_div(block_offset + data.len(), BLOCK_SIZE);

        for i in 0..blocks_to_act_on {
            // Resetting the block offset after first pass since we want to read from
            // the beginning of the block after the first time.
            if block_offset != 0 && i > 0 { block_offset = 0 };

            // Need to account for offsets from first and last blocks
            let num_bytes = if i == blocks_to_act_on - 1 {
                data.len() - read
            } else {
                BLOCK_SIZE - block_offset
            };

            let page = self.get_page(start + i);
            let pg_offset = self.fs.data_base + page * BLOCK_SIZE;
            let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size as usize, 0);
            &self.fs.device.read(&mut read_buf, pg_offset, BLOCK_SIZE);
            let disk_page = read_buf.read_bytes();
            // TODO: check compatability here
            let slice = &mut data[read..(read + num_bytes)];
            // read += slice.copy_from(page.slice(block_offset,
            // block_offset + num_bytes));
            unsafe {
                // copy_from is extremely slow! use copy_memory instead
                let src = disk_page[block_offset..(block_offset + num_bytes)].as_ptr();
                copy_nonoverlapping(src, slice.as_mut_ptr(), num_bytes);
            }

            read += num_bytes;
        }

        read
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn stat(&self) -> (Timespec, Timespec, Timespec) {
        (self.create_time, self.access_time, self.mod_time)
    }
}

#[cfg(test)]
mod tests {
    use super::mem;

    extern crate libc;

    use std::ffi::{CString, CStr};
    use std::ptr;
    use std::os::raw::{c_void, c_char, c_int};
    use inode::spdk_rs::raw;


    #[derive(Debug)]
    struct hello_context_t {
        bdev: *mut raw::spdk_bdev,
        bdev_desc: *mut raw::spdk_bdev_desc,
        bdev_io_channel: *mut raw::spdk_io_channel,
        buff: *mut c_char,
        bdev_name: *const c_char,
    }

    extern "C" fn read_complete(bdev_io: *mut raw::spdk_bdev_io,
                                success: bool,
                                cb_arg: *mut c_void) {
        let hello_context: *mut hello_context_t = cb_arg as *mut hello_context_t;

        unsafe {
            match success {
                true => {
                    let slice = CStr::from_ptr((*hello_context).buff);
                    println!("string buffer size without nul terminator: {}", slice.to_bytes().len());
                    println!("Read string from bdev: {}", CStr::from_ptr((*hello_context).buff).to_str().unwrap());
                }
                false => {
                    println!("bdev io read error");
                }
            }

            raw::spdk_bdev_free_io(bdev_io);
            raw::spdk_put_io_channel((*hello_context).bdev_io_channel);
            raw::spdk_bdev_close((*hello_context).bdev_desc);
            println!("Stopping app");
            raw::spdk_app_stop(if success { 0 } else { -1 });
        }
    }


    extern "C" fn write_complete(bdev_io: *mut raw::spdk_bdev_io,
                                 success: bool,
                                 cb_arg: *mut c_void) {
        let hello_context: *mut hello_context_t = cb_arg as *mut hello_context_t;
        let rc: c_int;
        let blk_size: u32;

        unsafe {
            raw::spdk_bdev_free_io(bdev_io);

            match success {
                true => {
                    println!("bdev io write completed successfully");
                }
                false => {
                    println!("bdev io write error: {}", raw::EIO);
                    raw::spdk_put_io_channel((*hello_context).bdev_io_channel);
                    raw::spdk_bdev_close((*hello_context).bdev_desc);
                    raw::spdk_app_stop(-1);
                    return;
                }
            }

            blk_size = raw::spdk_bdev_get_block_size((*hello_context).bdev);
            raw::memset((*hello_context).buff as *mut c_void, 0, blk_size as usize);

            println!("Reading io");
            let hello_context_ptr: *mut c_void = hello_context as *mut _ as *mut c_void;
            rc = raw::spdk_bdev_read((*hello_context).bdev_desc,
            (*hello_context).bdev_io_channel,
            (*hello_context).buff as *mut c_void,
            0,
            blk_size as u64,
            Some(read_complete),
            hello_context_ptr);
            if rc != 0 {
                println!("{} error while reading from bdev: {}", CStr::from_ptr(raw::spdk_strerror(-rc)).to_str().unwrap(), rc);
                raw::spdk_put_io_channel((*hello_context).bdev_io_channel);
                raw::spdk_bdev_close((*hello_context).bdev_desc);
                raw::spdk_app_stop(-1);
                return;
            }
        }
    }

    extern "C" fn hello_start(_arg1: *mut c_void, _arg2: *mut c_void) {
        let hello_context: *mut hello_context_t = _arg1 as *mut hello_context_t;
        let blk_size: u32;
        let buf_align: usize;
        let mut rc: c_int;
        unsafe { (*hello_context).bdev = ptr::null_mut(); }
        unsafe { (*hello_context).bdev_desc = ptr::null_mut(); }

        println!("Successfully started the application");

        unsafe {
            println!("Try to get a list of bdev ... ");
            let mut first: *mut raw::spdk_bdev = raw::spdk_bdev_first();
            while !first.is_null() {
                let owned_fmt = CString::new("bdev name: %s\n").unwrap();
                let fmt: *const c_char = owned_fmt.as_ptr();
                libc::printf(fmt, (*first).name);
                first = raw::spdk_bdev_next(first);
            }

            (*hello_context).bdev = raw::spdk_bdev_get_by_name((*hello_context).bdev_name);
            if (*hello_context).bdev.is_null() {
                println!("Could not find the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
                raw::spdk_app_stop(-1);
                return;
            }

            println!("Opening the bdev {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
            rc = raw::spdk_bdev_open((*hello_context).bdev, true, None, ptr::null_mut(), &mut (*hello_context).bdev_desc);
            if rc != 0 {
                println!("Could not open bdev: {}", CStr::from_ptr((*hello_context).bdev_name).to_str().unwrap());
                raw::spdk_app_stop(-1);
                return;
            }

            println!("Opening io channel");
            (*hello_context).bdev_io_channel = raw::spdk_bdev_get_io_channel((*hello_context).bdev_desc);
            if (*hello_context).bdev_io_channel.is_null() {
                println!("Could not create bdev I/O channel!!");
                raw::spdk_bdev_close((*hello_context).bdev_desc);
                raw::spdk_app_stop(-1);
                return;
            }

            blk_size = raw::spdk_bdev_get_block_size((*hello_context).bdev);
            buf_align = raw::spdk_bdev_get_buf_align((*hello_context).bdev);
            (*hello_context).buff = raw::spdk_dma_zmalloc(blk_size as usize, buf_align, ptr::null_mut()) as *mut c_char;
            if (*hello_context).buff.is_null() {
                println!("Failed to allocate buffer");
                raw::spdk_put_io_channel((*hello_context).bdev_io_channel);
                raw::spdk_bdev_close((*hello_context).bdev_desc);
                raw::spdk_app_stop(-1);
                return;
            }

            let owned_fmt = CString::new("%s\n").unwrap();
            let fmt: *const c_char = owned_fmt.as_ptr();
            let owned_content = CString::new("Hello World!\n").unwrap();
            let content: *const c_char = owned_content.as_ptr();
            raw::snprintf((*hello_context).buff, blk_size as usize, fmt, content);

            println!("Writing to the bdev");
            let hello_context_ptr: *mut c_void = hello_context as *mut _ as *mut c_void;
            rc = raw::spdk_bdev_write((*hello_context).bdev_desc, (*hello_context).bdev_io_channel,
            (*hello_context).buff as *mut c_void, 0, blk_size as u64, Some(write_complete), hello_context_ptr);
            if rc != 0 {
                println!("{0} error while writing to bdev: {1}", CStr::from_ptr(raw::spdk_strerror(-rc)).to_str().unwrap(), rc);
                raw::spdk_bdev_close((*hello_context).bdev_desc);
                raw::spdk_put_io_channel((*hello_context).bdev_io_channel);
                raw::spdk_app_stop(-1);
                return;
            }
        }
    }

    #[test]
    fn test_main() {
        println!("Enter test_main");

        unsafe {
            let mut opts: raw::spdk_app_opts;
            opts = mem::uninitialized();
            raw::spdk_app_opts_init(&mut opts);

            let mut hello_context: hello_context_t = mem::uninitialized();

            let owned_name = CString::new("hello_bdev").unwrap();
            opts.name = owned_name.as_ptr();

            let owned_config_file = CString::new("/home/zeyuanhu/rustfs/examples/hello_nvme_bdev/bdev.conf").unwrap();
            opts.config_file = owned_config_file.as_ptr();

            let owned_bdev_name = CString::new("Nvme0n1").unwrap();
            hello_context.bdev_name = owned_bdev_name.as_ptr();

            let hello_context_ptr: *mut c_void = &mut hello_context as *mut _ as *mut c_void;
            let rc: c_int = raw::spdk_app_start(&mut opts, Some(hello_start), hello_context_ptr, ptr::null_mut());
            if rc != 0 {
                panic!("ERROR starting application");
            }

            raw::spdk_dma_free(hello_context.buff as *mut c_void);

            raw::spdk_app_fini();
        }
    }
}
