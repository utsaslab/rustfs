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
    pub buf_align: usize,
    blk_size: usize,
}

impl Device {
    pub fn new() -> Device {
        let mut first_bdev = bdev::first();
        while !first_bdev.is_none() {
            let bdev = first_bdev.unwrap();
            println!("bdev name: {}", bdev.name());
            first_bdev = bdev::next(&bdev);
        }

        let ret = bdev::get_by_name("Malloc0");
        let bdev = ret.unwrap();
        let mut desc = SpdkBdevDesc::new();

        match bdev::open(bdev.clone(), true, &mut desc) {
            Ok(_) => println!("Successfully open the device"),
            _ => {}
        }
    }
    // nbytes = blk_size?
    pub fn read(&self, read_buf: &mut env::Buf, offset: usize, nbytes: usize) -> Result<usize, Error> {
        await!(bdev::read(
            self.desc.clone(),
            &self.io_channel,
            &mut read_buf,
            offset,
            nbytes
        ))
    }

    pub fn write(&self, write_buf: &env::Buf, offset: usize, nbytes: usize) -> Result<usize, Error> {
        await!(bdev::write(
            self.desc.clone(),
            &self.io_channel,
            write_buf,
            offset,
            nbytes
        ))
    }

    pub fn blk_size(&self) -> usize {
        self.blk_size
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        spdk_rs::thread::put_io_channel(self.io_channel);
        bdev::close(self.desc);
        spdk_rs::event::app_stop(true);
    }
}
