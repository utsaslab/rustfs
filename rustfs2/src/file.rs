use self::File::{DataFile, Directory};

use crate::constants;
use crate::inode;

use constants::DIR_TYPE;
use inode::Inode;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

//pub type RcDirContent<'r> = Rc<RefCell<Box<DirectoryContent<'r>>>>;
//pub type RcInode = Rc<RefCell<Box<Inode>>>;

// File is a thing wrapper around Inodes and Directories. The whole point is to
// provide a layer of indirection. FileHandle's and Directory entries, then,
// point to these guys instead of directly to Inodes/Directories
#[derive(Clone)]
pub enum File {
    DataFile(Inode),
    Directory(DirectoryContent),
    EmptyFile,
}

#[derive(Clone)]
pub struct FileHandle {
    file: File,
    seek: Cell<usize>,
}

// Preserve this and write to disk?
#[derive(Clone)]
pub struct DirectoryContent {
    pub entries: Option<HashMap<&'static str, File>>,
    pub inode: Inode,
}

pub enum Whence {
    SeekSet,
    SeekCur,
    SeekEnd,
}

impl File {
    pub fn new_dir(_parent: Option<File>) -> File {
        // let content = DirectoryContent {
        //     entries: HashMap::new(),
        //     inode: Inode { fs, DIR_TYPE, inum },
        // };
        // //        let rc = Rc::new(RefCell::new(content));
        // let dir = Directory(content);
        // // TODO: write to disk here ??

        // // Note that dir is RCd, so this is cheap
        // // Used to borrow dir and mut_dir at "same time"
        // // RefCell makes sure we're not doing anything wrong
        // // let mut mut_dir = dir.clone();

        // // // Setting up "." and ".."
        // // mut_dir.insert(".", dir.clone());
        // // match parent {
        // //   None => mut_dir.insert("..", dir.clone()),
        // //   Some(f) => mut_dir.insert("..", f)
        // // }

        // dir
        unimplemented!();
    }

    pub fn new_data_file(inode: Inode) -> File {
        // TODO: write to disk ??
        DataFile(inode);
        unimplemented!();
    }

    pub fn get_dir_inode(&self) -> Inode {
        match self.clone() {
            Directory(dir_content) => dir_content.inode,
            _ => panic!("not a directory"),
        }
    }

    pub fn get_inode(&self) -> Inode {
        match self.clone() {
            DataFile(inode) => inode,
            _ => panic!("not a directory"),
        }
    }
}

impl FileHandle {
    // Probably not the right type.
    pub fn new(file: File) -> FileHandle {
        FileHandle {
            file: file,
            seek: Cell::new(0),
        }
    }

<<<<<<< HEAD
    pub async fn read<'a>(&'a self, dst: &'a mut [u8]) -> usize {
=======
    pub async fn read(&self, dst: &mut [u8]) -> usize {
>>>>>>> a17418ea523c0af527c906ebcbf965b099e88a03
        let offset = self.seek.get();
        let inode = self.file.get_inode();
        let changed = await!(inode.read(offset, dst));
        self.seek.set(offset + changed);
        changed
    }

<<<<<<< HEAD
    pub async fn write<'a>(&'a mut self, src: &'a [u8]) -> usize {
=======
    pub async fn write(&mut self, src: &[u8]) -> usize {
>>>>>>> a17418ea523c0af527c906ebcbf965b099e88a03
        let offset = self.seek.get();
        let inode = self.file.get_inode();
        let changed = await!(inode.write(offset, src));
        self.seek.set(offset + changed);
        changed
    }

    pub fn seek(&mut self, offset: isize, whence: Whence) -> usize {
        // let seek = self.seek.get();
        // let new_seek = match whence {
        //     Whence::SeekSet => offset as usize,
        //     Whence::SeekCur => (seek as isize + offset) as usize,
        //     Whence::SeekEnd => (inode_rc.borrow().size() as isize + offset) as usize,
        // };

        // self.seek.set(new_seek);
        // new_seek
        unimplemented!();
    }
}
