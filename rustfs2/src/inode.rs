use crate::constants::{BLOCK_SIZE, INODE_SIZE, LIST_SIZE};
use crate::fs::{fs_internal, FsInternal};
use std::mem;
use std::ptr;
use std::ptr::copy_nonoverlapping;
use crate::file::{File, DirectoryContent, File::Directory, File::DataFile};
use time;
use time::Timespec;

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
    for x in list.iter_mut() {
        unsafe {
            ptr::write(x, None);
        }
    }
    list
}

#[derive(Clone)]
pub struct Inode {
    pub inum: usize,
    pub dirtype: usize,
    single: Option<usize>,
    double: Option<usize>,
    size: usize,
}

impl Inode {
    pub fn new(dirtype: usize, inum: usize) -> Inode {
        Inode {
            inum: inum,
            dirtype: dirtype,
            single: None,
            double: None,
            size: 0,
        }
    }

    async fn read_inode(&self) {
        let fs = fs_internal.into_inner();
        let offset = fs.inode_base + self.inum * INODE_SIZE;
        let blk_size = fs.device.blk_size();
        let blk = offset / blk_size;
        let blk_offset = offset % blk_size;
        let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, 0);
        await!(fs.device.read(&mut read_buf, blk, blk_size));
        let mut buf = read_buf.read_bytes(blk_size);
        let mut content = &buf[blk_offset..blk_offset + INODE_SIZE];

        self.dirtype = mem::transmute::<[u8; 8], usize>(*array_ref![content, 0, 8]);
        self.size = mem::transmute::<[u8; 8], usize>(*array_ref![content, 8, 8]);
        self.single = Some(mem::transmute::<[u8; 8], usize>(*array_ref![
            content, 16, 8
        ]));
        self.double = Some(mem::transmute::<[u8; 8], usize>(*array_ref![
            content, 24, 8
        ]));
    }

    pub async fn read_file_from_inum(inum: usize) -> File {
        let fs = fs_internal.borrow();
        let blk_size = fs.device.blk_size();
        let offset = fs.inode_base + inum * INODE_SIZE;

        let blk = offset / blk_size;
        let blk_offset = offset % BLOCK_SIZE;        
        let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, fs.device.buf_align());
        await!(fs.device.read(&mut read_buf, blk, blk_size));
        let buf = read_buf.read_bytes(blk_size);
        let mut content = &buf[blk_offset..blk_offset + INODE_SIZE];
        let inode:Inode;
        unsafe {
            let dirtype = mem::transmute::<[u8; 8], usize>(*array_ref![content, 0, 8]);
            let size = mem::transmute::<[u8; 8], usize>(*array_ref![content, 8, 8]);
            let single = mem::transmute::<[u8; 8], usize>(*array_ref![content, 16, 8]);
            let double = mem::transmute::<[u8; 8], usize>(*array_ref![content, 24, 8]);
            inode = Inode {
                inum: inum,
                dirtype: dirtype,
                size: size,
                single: Some(single),
                double: Some(double),
            };
            match dirtype{
                DIR_TYPE => { 
                    let dir_content = DirectoryContent{
                        entries: None,
                        inode: inode,
                    };
                    Directory(dir_content)
                },
                FILE_TYPE => DataFile(inode),
                _ => panic!("unknown dirtype {}", dirtype)
            }                        
        }
    }

    fn parse_entry(raw_read: &[u8], index: usize) -> usize {
        let start = index * 8;
        let content = &raw_read[start..start + 8];
        let entry: usize;
        unsafe {
            entry = mem::transmute::<[u8; 8], usize>(*array_ref![content, 0, 8]);
        }
        entry
    }

    async fn write_inode(&self) {
        // TODO: add unit test
        let fs = fs_internal.borrow();
        let blk_size = fs.device.blk_size();
        let offset = fs.inode_base + self.inum * INODE_SIZE;
        let blk = offset / blk_size;
        let blk_offset = offset % blk_size;
        let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, 0);
        await!(fs.device.read(&mut read_buf, blk, blk_size));
        let mut buf = read_buf.read_bytes(blk_size);
        let mut content = &buf[blk_offset..blk_offset + INODE_SIZE];
        let tmp = mem::transmute::<usize, [u8; 8]>(self.dirtype);
        content[0..8].copy_from_slice(&tmp[0..8]);
        let tmp = mem::transmute::<usize, [u8; 8]>(self.size);
        content[8..16].copy_from_slice(&tmp[0..8]);
        let tmp = mem::transmute::<usize, [u8; 8]>(self.single.unwrap());
        content[16..24].copy_from_slice(&tmp[0..8]);
        let tmp = mem::transmute::<usize, [u8; 8]>(self.double.unwrap());
        content[24..32].copy_from_slice(&tmp[0..8]);
        let mut write_buf = read_buf;
        write_buf.fill_bytes(buf);
        await!(fs.device.write(&write_buf, blk, blk_size));
    }

    /// read inode metadata and return block number
    async fn get_or_alloc_page(&mut self, num: usize) -> usize {
        unsafe {
        let fs = fs_internal.borrow();
        let blk_size = fs.device.blk_size();
        if num >= LIST_SIZE + 1 {
            panic!("Maximum file size exceeded!")
        };

        let mut need_update: bool = false;
        await!(self.read_inode());

        // Getting a pointer to the page
        let page = if num == 0 {
            if self.single.is_none() {
                //                if self.size == 0 {
                self.single = Some(await!(FsInternal::alloc_block()));
                need_update = true;
                //                }else{
                //                    &mut self.read_inode();
                //                }
            }
        self.single.unwrap()
        } else {
            // if the page num is in the doubly-indirect list. We allocate a new
            // entry list where necessary (*entry_list = ...)
            let index = num - 1;
            if self.double.is_none() {
                //                if self.size <= blk_size {
                self.double = Some(await!(FsInternal::alloc_block()));
                need_update = true;
                //                }else{
                //                }
            }
            let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, 0);
            let offset = fs.data_base + self.double.unwrap() * blk_size;
            await!(fs.device.read(&mut read_buf, offset, blk_size));
            let entry = Inode::parse_entry(read_buf.read_bytes(blk_size), index);
            entry
        };

        if need_update {
            self.write_inode();
        }
            page
        }
    }

    async fn get_page(&self, num: usize) -> usize {

        let fs = fs_internal.into_inner();
        let blk_size = fs.device.blk_size();
        if num * blk_size >= self.size {
            panic!("Page does not exist.")
        };
        await!(self.read_inode());
        if num == 0 {
            0
        } else {
            let index = num - 1;
            let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, 0);
            let offset = fs.data_base + self.double.unwrap() * blk_size;
            await!(fs.device.read(&mut read_buf, offset, blk_size));
            let entry = Inode::parse_entry(read_buf.read_bytes(blk_size), index);
            entry
            // TODO: read the indirect block
        }
    }

    pub async fn write<'a>(&'a mut self, offset: usize, data: &'a [u8]) -> usize {
        unsafe {
        let fs = fs_internal.borrow();
        let blk_size = fs.device.blk_size();

        let mut written: usize = 0;
        let mut block_offset = offset % blk_size; // offset from first block

        let start = offset / blk_size; // first block to act on
        let blocks_to_act_on = ceil_div(block_offset + data.len(), blk_size);

        for i in 0..blocks_to_act_on {
            // Resetting the block offset after first pass since we want to read from
            // the beginning of the block after the first time.
            if block_offset != 0 && i > 0 {
                block_offset = 0
            };

            // Need to account for offsets from first and last blocks
            let num_bytes = if i == blocks_to_act_on - 1 {
                data.len() - written
            } else {
                blk_size - block_offset
            };

            // Finding our block, writing to it
            let page = await!(self.get_or_alloc_page(start + i));

            // TODO: check this!
            let pg_offset = fs.data_base + page * blk_size;
            let mut read_buf = spdk_rs::env::dma_zmalloc(blk_size, 0);
            await!(fs.device.read(&mut read_buf, pg_offset, blk_size));
            let disk_page = read_buf.read_bytes(blk_size);
            // let slice = array_mut_ref![disk_page, block_offset, num_bytes];
            let slice = &disk_page[block_offset..(block_offset + num_bytes)];
            // written += slice.copy_from(data.slice(written, written + num_bytes));
            //TODO: comment it out due to compilation error
            // unsafe {
            //     // TODO: This may be extremely slow! Use copy_nonoverlapping, perhaps.
            //     let src = data[written..(written + num_bytes)].as_ptr();
            //     copy_nonoverlapping(src, slice.as_mut_ptr(), num_bytes);
            // }
            // END TODO
            let mut write_buf = spdk_rs::env::dma_zmalloc(BLOCK_SIZE, 0);
            write_buf.fill_bytes(disk_page);
            await!(fs.device.write(&mut write_buf, offset, blk_size));
            written += num_bytes;
        }
        let last_byte = offset + written;
        if self.size < last_byte {
            self.size = last_byte;
        }
        //        let time_now = time::get_time();
        //        self.mod_time = time_now;
        //        self.access_time = time_now;
            written
        }
    }

    pub async fn read<'a>(&'a self, offset: usize, data: &'a mut [u8]) -> usize {
        let fs = fs_internal.into_inner();
        let blk_size = fs.device.blk_size();
        let mut read = 0;
        let mut block_offset = offset % blk_size; // offset from first block
        let start = offset / blk_size; // first block to act on
        let blocks_to_act_on = ceil_div(block_offset + data.len(), blk_size);

        for i in 0..blocks_to_act_on {
            // Resetting the block offset after first pass since we want to read from
            // the beginning of the block after the first time.
            if block_offset != 0 && i > 0 {
                block_offset = 0
            };

            // Need to account for offsets from first and last blocks
            let num_bytes = if i == blocks_to_act_on - 1 {
                data.len() - read
            } else {
                blk_size - block_offset
            };

            let page = await!(self.get_page(start + i));
            let pg_offset = fs.data_base + page * blk_size;
            let mut read_buf = spdk_rs::env::dma_zmalloc(fs.device.blk_size(), 0);
            await!(fs.device.read(&mut read_buf, pg_offset, blk_size));
            let disk_page = read_buf.read_bytes(blk_size);
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
}
