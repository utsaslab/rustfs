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
        let ret = bdev::get_by_name("Malloc0");
        let bdev = ret.unwrap();
        let mut desc = SpdkBdevDesc::new();

        match bdev::open(bdev.clone(), true, &mut desc) {
            Ok(_) => println!("Successfully open the device"),
            _ => {},
        };
        let io_channel = spdk_rs::bdev::get_io_channel(desc.clone()).unwrap();
        let blk_size = spdk_rs::bdev::get_block_size(bdev.clone());
        let buf_align = spdk_rs::bdev::get_buf_align(bdev.clone());
        Device {
            desc: desc,
            io_channel: io_channel,
            buf_align: buf_align as usize,
            blk_size: blk_size as usize,
        }
    }

    pub async fn read<'a>(&'a self, read_buf: &'a mut env::Buf, offset: usize, nbytes: usize) -> Result<(), Error> {
        match await!(bdev::read(
                self.desc.clone(),
                &self.io_channel,
                read_buf,
                offset as u64,
                nbytes as u64
        )) {
            Ok(_) => {}
            Err(error) => panic!("{:?}", error),
        };
        Ok(())
    }

    pub async fn write<'a>(&'a self, write_buf: &'a env::Buf, offset: usize, nbytes: usize) -> Result<(), Error> {
        match await!(bdev::write(
                self.desc.clone(),
                &self.io_channel,
                &write_buf,
                offset as u64,
                nbytes as u64
        )) {
            Ok(_) => {}
            Err(error) => panic!("{:?}", error),
        }
        Ok(())
    }

    pub fn blk_size(&self) -> usize {
        self.blk_size
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        spdk_rs::thread::put_io_channel(self.io_channel.clone());
        bdev::close(self.desc.clone());
        spdk_rs::event::app_stop(true);
    }
}
