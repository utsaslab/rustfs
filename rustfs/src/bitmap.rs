use crate::constants;
use crate::device;
use crate::file;
use crate::inode;

use constants::{BLOCK_SIZE, DIR_TYPE, INODE_SIZE};
use device::Device;
use failure::Error;
use file::File;
use inode::Inode;

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
    root: Option<File>,
}

impl FS {
    pub fn new() -> FS {
        let device = Device::new();
        let blk_size = device.blk_size();
        FS {
            device: device,
            inode_base: 3 * blk_size,
            data_base: 3 * blk_size + INODE_SIZE * blk_size * 8,
            inode_bitmap: Bitmap::new(blk_size, blk_size),
            data_bitmap: Bitmap::new(2 * blk_size, blk_size),
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
        write_buf.fill(&self.device.blk_size as usize, "%s", "RustFS--");
        &mut self.device.write(&write_buf, 0, &self.device.blk_size);

        // Define - root lives in first inode
        let byte: u8 = 1;
        write_buf.fill(&self.device.blk_size as usize, "%s", byte.to_string());
        &mut self
            .device
            .write(&write_buf, &self.device.blk_size, &self.device.blk_size);
        &mut self
            .device
            .write(&zero_buf, 2 * &self.device.blk_size, &self.device.blk_size);
        let root_inode = inode::Inode::new(&mut self, DIR_TYPE, 0);
        root_inode.get_or_alloc_page(0);
        root_inode.write_inode();
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
        self.inode_bitmap.bitmap = read_buf.read();
        &mut self
            .device
            .read(&read_buf, 2 * &self.device.blk_size, &self.device.blk_size);
        self.data_bitmap.bitmap = read_buf.read();
        &mut self
            .device
            .read(&read_buf, 3 * &self.device.blk_size, &self.device.blk_size);
        let root_inode:Inode;
        root_inode.read_inode();
    }

    fn make_root(&self, root_inode: Inode) {
        let dir_content = DirectoryContent {
            entries: HashMap::new(),
            
        };
         File::Directory(&self.root.unwrap())
    }
    /*
    pub fn find(String path){

    }
    */
}
