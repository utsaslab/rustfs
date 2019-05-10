use crate::constants;
use crate::device;
use crate::file;
use crate::inode;

use constants::{BLOCK_SIZE, DIR_TYPE, INODE_SIZE};
use crate::file::DirectoryContent;
use device::Device;
use failure::Error;
use file::File;
use file::File::Directory;
use inode::Inode;

#[derive(Debug, Fail)]
pub enum BitmapErr {
    #[fail(display = "Bitmap full")]
    Full(),
}

pub struct Bitmap {
    bitmap: Vec<u8>,
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
}

pub struct FS<'r> {
    pub device: Device,
    //    data_bitmap_base: usize,
    //    inode_bitmap_base: usize,
    pub inode_base: usize,
    pub data_base: usize,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    pub root: Option<File<'r>>,
}

impl<'r> FS<'r> {
    pub fn new() -> FS<'r> {
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
        let index = &self.data_bitmap.find().unwrap();
        let offset = index * &self.device.blk_size() + &self.data_base;
        let zero_buf =
            spdk_rs::env::dma_zmalloc(self.device.blk_size(), self.device.buf_align);
        &mut self
            .device
            .write(&zero_buf, offset, self.device.blk_size());
        offset / self.device.blk_size()
    }

    pub fn mkfs(&'r mut self) {
        let zero_buf =
            spdk_rs::env::dma_zmalloc(self.device.blk_size(), self.device.buf_align);
        let mut write_buf =
            spdk_rs::env::dma_zmalloc(self.device.blk_size(), self.device.buf_align);
        write_buf.fill(self.device.blk_size(), "%s", "RustFS--");
        &mut self.device.write(&write_buf, 0, self.device.blk_size());

        // Define - root lives in first inode
        let byte:[u8;1] = [1;1];
        write_buf.fill_bytes(&byte[..]);
        &mut self
            .device
            .write(&write_buf, self.device.blk_size(), self.device.blk_size());
        &mut self
            .device
            .write(&zero_buf, 2 * self.device.blk_size(), self.device.blk_size());
        let mut root_inode = inode::Inode::new(&mut self, DIR_TYPE, 0);
        root_inode.get_or_alloc_page(0);
        root_inode.write_inode();
        self.make_root(root_inode);
    }

    pub fn mount(&mut self) {
        let mut read_buf =
            spdk_rs::env::dma_zmalloc(self.device.blk_size(), self.device.buf_align);
        &mut self
            .device
            .read(&mut read_buf, self.device.blk_size(), self.device.blk_size());
        self.inode_bitmap.bitmap.copy_from_slice(read_buf.read_bytes(self.device.blk_size()));
        &mut self
            .device
            .read(&mut read_buf, 2 * self.device.blk_size(), self.device.blk_size());
        self.data_bitmap.bitmap.copy_from_slice(read_buf.read_bytes(self.device.blk_size()));
        &mut self
            .device
            .read(&mut read_buf, 3 * self.device.blk_size(), self.device.blk_size());
        let root_inode:Inode;
        root_inode.read_inode();
        self.make_root(root_inode);
    }

    fn make_root(&mut self, root_inode: Inode<'r>) {
        let dir_content = DirectoryContent {
            entries: None,
            inode: root_inode,
        };
        self.root = Some(Directory(dir_content));
    }
    /*
    pub fn find(String path){

    }
    */
}
