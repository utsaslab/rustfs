/*************************************************************************
  > File Name:       device.rs
  > Author:          Yuhao Zheng
  > Mail:            yuhao@utexas.edu
  > Created Time:    4/20/19
  > Description:
           
    This file contains the implementation of bitmap.
************************************************************************/

extern crate spdk_rs;

use spdk_rs::bdev;
use spdk_rs::bdev::SpdkBdevDesc;
use spdk_rs::thread::SpdkIoChannel;

pub struct Device{
    desc: mut SpdkBdevDesc,
    io_channel: SpdkIoChannel,
    buf_align: usize,
    blk_size: u32,
}

impl Device{
    pub fn new() -> Device{
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
    fn read(&self, read_buf: &mut env::Buf, offset: u64, 
            nbytes: u64) -> Result<usize, Error> {
        await!(bdev::read(self.desc.clone(), &self.io_channel, 
                          &mut read_buf, offset, nbytes));
    }

    fn write(&self, write_buf: &mut env::Buf, offset: u64, 
             nbytes: u64) -> Result<usize, Error> {
        await!(bdev::write(self.desc.clone(), &self.io_channel, 
                          &mut write_buf, offset, nbytes));
    }
}

impl Drop for Device{
    fn drop(&mut self){
        spdk_rs::thread::put_io_channel(io_channel);
        bdev::close(desc);
        spdk_rs::event::app_stop(true);
    }
}
