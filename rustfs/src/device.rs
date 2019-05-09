//! A wrapper for device, which an abstraction for the underlying SPDK objects
extern crate spdk_rs;

use failure::Error;
use spdk_rs::bdev;
use spdk_rs::bdev::SpdkBdevDesc;
use spdk_rs::env;
use spdk_rs::thread::SpdkIoChannel;

pub struct Device {
    desc: SpdkBdevDesc,
    io_channel: SpdkIoChannel,
    buf_align: usize,
    blk_size: u32,
}

impl Device {
    pub fn new() -> Device {
        let ret = bdev::get_by_name("Malloc0");
        let bdev = ret.unwrap();
        let mut desc = SpdkBdevDesc::new();

        match bdev::open(bdev.clone(), true, &mut desc) {
            Ok(_) => println!("Successfully open the device"),
            _ => {}
        }
        let io_channel = bdev::get_io_channel(desc.clone()).unwrap();
        let blk_size = spdk_rs::bdev::get_block_size(bdev.clone());
        let buf_align = spdk_rs::bdev::get_buf_align(bdev.clone());
        Device {
            desc,
            io_channel,
            buf_align,
            blk_size,
        }
    }
    // nbytes = blk_size?
    pub fn read(&self, read_buf: &mut env::Buf, offset: u64, nbytes: u64) -> Result<usize, Error> {
        match await!(bdev::read(
            self.desc.clone(),
            &self.io_channel,
            &mut read_buf,
            offset,
            nbytes
        )) {
            Ok(_) => {
                let read_buf_string = read_buf.read().to_string();
                Ok(read_buf_string.len())
            }
            Err(error) => panic!("{:}", error),
        }
    }

    pub fn write(&self, write_buf: &env::Buf, offset: u64, nbytes: u64) -> Result<usize, Error> {
        await!(bdev::write(
            self.desc.clone(),
            &self.io_channel,
            write_buf,
            offset,
            nbytes
        ))
    }

    pub fn blk_size(&self) -> usize {
        self.blk_size as usize
    }

    pub fn buf_align(&self) -> usize {
        self.buf_align
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        spdk_rs::thread::put_io_channel(self.io_channel);
        bdev::close(self.desc);
        spdk_rs::event::app_stop(true);
    }
}
