use crate::constants::{BLOCK_SIZE, DIR_TYPE, INODE_SIZE};
use crate::device::Device;
use crate::file::{DirectoryContent, File, File::Directory};
use crate::fs::{fs_internal, FsInternal};
use crate::inode::Inode;
use failure::Error;

#[derive(Debug, Fail)]
pub enum BitmapErr {
    #[fail(display = "Bitmap full")]
    Full(),
}

pub struct Bitmap {
    pub bitmap: Vec<u8>,
    offset: usize, // base offset on SSD
}

impl Bitmap {
    pub fn new(offset: usize, size: usize) -> Bitmap {
        Bitmap {
            bitmap: vec![0; size],
            offset: offset,
        }
    }

    pub fn set(&mut self, index: usize) {
        let bit_index = index % 8;
        let byte_offset = index / 8;
        let byte = &mut self.bitmap[byte_offset];
        *byte |= 1 << bit_index;
    }
    pub fn clear(&mut self, index: usize) {
        let bit_index = index % 8;
        let byte_offset = index / 8;
        let byte = &mut self.bitmap[byte_offset]; // is this the correct way to specify    pointer?
        *byte &= !(1 << bit_index);
    }

    // find and set
    pub fn find(&mut self) -> Result<usize, Error> {
        for byte in &mut self.bitmap {
            if !(*byte) != 0 {
                let mut mask = 1;
                for i in 0..8 {
                    if *byte & mask == 0 {
                        *byte |= mask;
                        return Ok(i + (*byte) as usize * 8);
                    }
                    mask *= 2;
                }
            }
        }
        println!("*** Bitmap full");
        Err(BitmapErr::Full())?
    }

    pub async fn read_bitmap(&mut self) {
        unsafe {
            let fs = fs_internal.borrow();
            let blk_size = fs.device.blk_size();
            let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, fs.device.buf_align());
            await!(fs.device.read(&mut read_buf, self.offset, blk_size));
            let mut buf = read_buf.read_bytes(blk_size);
            self.bitmap.copy_from_slice(&buf[0..blk_size]);
        }
    }

    pub async fn write_bitmap(&self) {
        unsafe {
            let fs = fs_internal.borrow();
            let blk_size = fs.device.blk_size();
            let mut write_buf = spdk_rs::env::dma_zmalloc(blk_size, fs.device.buf_align());
            write_buf.fill_bytes(&self.bitmap[0..blk_size]);
            await!(fs.device.write(&write_buf, self.offset, blk_size));
        }
    }
}
