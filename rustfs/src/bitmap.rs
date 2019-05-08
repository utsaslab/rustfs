/*************************************************************************
  > File Name:       bitmap.rs
  > Author:          Yuhao Zheng
  > Mail:            yuhao@utexas.edu
  > Created Time:    4/20/19
  > Description:

    This file contains the implementation of bitmap.
************************************************************************/

use crate::device;

use device::Device;
use failure::Error;

#[derive(Debug, Fail)]
pub enum BitmapErr {
    #[fail(display = "Bitmap full")]
    Full(),
}

pub struct Bitmap {
    bitmap: Vec<u8>,
    offset: u64, // base offset on SSD
}

impl Bitmap {
    pub fn new(offset: u64, size: usize) -> Bitmap {
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
    fn find(&mut self) -> Result<usize, Error> {
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
}

pub struct FS {
    device: Device,
    //    data_bitmap_base: usize,
    //    inode_bitmap_base: usize,
    inode_base: usize,
    data_base: usize,
    inode_bitmap: Bitmap,
    data_bitmap: Bitmap,
    root: Option<Inode>,
}

impl FS {
    pub fn new() -> FS {
        device = Device::new();
        inode_bmp = Bitmap::new(device.blk_size, device.blk_size);
        data_bmp = Bitmap::new(2 * device.blk_size, device.blk_size);
        FS {
            device: device,
            inode_base: 3 * device.blk_size,
            data_base: 3 * device.blk_size + inode::size() * device.blk_size * 8,
            inode_bitmap: inode_bmp,
            data_bitmap: data_bmp,
            root: None,
        }
    }

    pub fn alloc_block(&mut self) -> usize {
        let index = &self.data_bitmap.find()?;
        let offset = index * &self.device.blk_size + &self.data_base;
        let zero_buf =
            spdk_rs::env::dma_zmalloc(&self.device.blk_size as usize, &self.device.buf_align);
        &mut self
            .device
            .write(&mut zero_buf, offset, &self.device.blk_size);
        offset / &self.device.blk_size
    }

    pub fn mkfs(&mut self) {
        let zero_buf =
            spdk_rs::env::dma_zmalloc(&self.device.blk_size as usize, &self.device.buf_align);
        let &mut write_buf =
            spdk_rs::env::dma_zmalloc(&self.device.blk_size as usize, &self.device.buf_align);
        write_buf.fill(blk_size as usize, "%s", "RustFS--");
        &mut self.device.write(&write_buf, 0, &self.device.blk_size);

        // Define - root lives in first inode
        let byte: u8 = 1;
        write_buf.fill(blk_size as usize, "%s", byte.to_string());
        &mut self
            .device
            .write(&write_buf, &self.device.blk_size, &self.device.blk_size);
        &mut self
            .device
            .write(&zero_buf, 2 * &self.device.blk_size, &self.device.blk_size);
        root = Inode(&mut self, DIR_TYPE, 0);
        write_buf.fill(blk_size as usize, "%s\n", root.to_string());
        &mut self
            .device
            .write(&write_buf, 3 * &self.device.blk_size, &self.device.blk_size);
    }

    pub fn mount(&mut self) {
        let &mut read_buf =
            spdk_rs::env::dma_zmalloc(&self.device.blk_size as usize, &self.device.buf_align);
        &mut self
            .device
            .read(&read_buf, &self.device.blk_size, &self.device.blk_size);
        inode_bitmap.bitmap = read_buf.read();
        &mut self
            .device
            .read(&read_buf, 2 * &self.device.blk_size, &self.device.blk_size);
        data_bitmap.bitmap = read_buf.read();
        &mut self
            .device
            .read(&read_buf, 3 * &self.device.blk_size, &self.device.blk_size);
        let inode = read_buf.read().to_string().parse::<Inode>().unwrap;
        root = inode;
    }

    pub fn root(&self) -> File {
        Datafile(&self.root.unwrap())
    }
    /*
    pub fn find(String path){

    }
    */
}
