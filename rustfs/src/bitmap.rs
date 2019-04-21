/*************************************************************************
  > File Name:       bitmap.rs
  > Author:          Yuhao Zheng
  > Mail:            yuhao@utexas.edu
  > Created Time:    4/20/19
  > Description:
           
    This file contains the implementation of bitmap.
************************************************************************/

#[macro_use]
extern crate failure;
extern crate spdk_rs;

use failure::Error;

#[derive(Debug, Fail)]
pub enum BitmapErr{
    #[fail(display = "Bitmap full")]
    Full(),
}

pub struct Bitmap{
    bitmap: Vec<u8>,
    offset: u64, // base offset on SSD
}

impl Bitmap{
    pub fn new(offset: u64, size: usize) -> Bitmap{
        Bitmap{
            bitmap: vec![0; size],
            offset: offset,
        }   
    }   

    pub fn set(&mut self, index: usize){
        let bit_index = index % 8;
        let byte_offset = index / 8;
        let byte = &mut self.bitmap[byte_offset];
        *byte |= 1 << bit_index;
    }
    pub fn clear(&mut self, index: usize){
        let bit_index = index % 8;
        let byte_offset = index / 8;
        let byte = &mut self.bitmap[byte_offset]; // is this the correct way to specify    pointer?
        *byte &= !(1 << bit_index);
    }

    fn find(&mut self) -> Result<usize, Error>{
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
        Err(BitmapErr::Full())?
    }
}

pub struct FS{
    device: Device,
//    data_bitmap_base: usize,
//    inode_bitmap_base: usize,
//    inode_base: usize,
    inode_bitmap: Bitmap,
    data_bitmap: Bitmap,
}

impl FS{
    pub fn new() -> FS{
        device = Device::new();
        inode_bmp = Bitmap::new(device.blk_size, device.blk_size);
        data_bmp = Bitmap::new(2*device.blk_size, device.blk_size);
        FS{
            device: device,
            inode_bitmap: inode_bmp,
            data_bitmap: data_bmp,
        }
    }
    
    fn alloc_block() -> usize{
    
    }

    fn mkfs(){
    
    }

    fn root(){
    
    }

    fn find(){
    
    }
}

