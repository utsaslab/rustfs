use crate::file;
use crate::inode::Inode;
use file::File;
use file::File::{Directory, DataFile};
use file::DirectoryContent;
use std::collections::HashMap;
use std::mem;
use std::str;
use std::string::String;

pub trait DirectoryHandle<'r>: Sized {
    fn is_dir(&self) -> bool;
    fn insert(&mut self, name: &'r str, file: Self);
    fn remove(&mut self, name: &'r str);
    fn get(&self, name: &'r str) -> Option<Self>;
    fn read_dir(&mut self);
    fn write_dir(&self);
}

impl<'r> DirectoryHandle<'r> for File<'r> {
    fn is_dir(&self) -> bool {
        match self {
            &Directory(_) => true,
            _ => false,
        }
    }

    fn insert(&mut self, name: &'r str, file: File<'r>) {
        let mut dc = match self{
             &mut Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        match dc.entries {
             None => { 
                 &self.read_dir();
                 dc = match self {
                    &mut Directory(dir_content) => dir_content,
                    _ => panic!("not a dir"),
                 }
             },
             _ => {},
         };
        // TODO: check whether name already exists
        dc.entries.unwrap().insert(name, file);
        self.write_dir();
    }

    fn remove(&mut self, name: &'r str) {
        let dc = match self{
             &mut Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        match dc.entries {
             None => { 
                 &self.read_dir();
                 dc = match self {
                    &mut Directory(dir_content) => dir_content,
                    _ => panic!("not a dir"),
                 }
             },
             _ => {},
         };
        dc.entries.unwrap().remove(&name);
        &self.write_dir();
        //        let rc = self.get_dir_rc();
        // let mut content = rc.borrow_mut();
        // content.entries.remove(&name);
        // unimplemented!();
    }

    fn get(&self, name: &'r str) -> Option<File<'r>> {
        let dc = match self{
             &Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        match dc.entries {
             None => { 
                 &self.read_dir();
                 dc = match self {
                    &Directory(dir_content) => dir_content,
                    _ => panic!("not a dir"),
                 }
             },
             _ => {},
         };

                  
//        match dc.entries.unwrap().get(&name) {
//             None => None,
        //             Some(ref file) => Some((*file).clone()), // It's RC
//             Some() =>
//         }
//      Is it correct to clone??   
        Some((*dc.entries.unwrap().get(&name).unwrap()).clone())
    }

    // read from disk
    fn read_dir(&mut self){
        let mut dc = match self{
             &mut Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        let entry_map = match dc.entries {
            None => HashMap::new(),
            Some(hm) => hm,
        };
        let data:Vec<u8> = vec![0; dc.inode.size()]; 
        let read_buf = &mut data[..];
        dc.inode.read(0, &mut read_buf);
        let iters:usize = dc.inode.size() / 128;
        for i in 0..(iters+1) {
            let inum: usize;
            let name: &str;
            let start = 128 * iters;
            unsafe{
                inum = mem::transmute::<[u8; 8], usize>(*array_ref![read_buf, start, 8]);
                name = str::from_utf8(&read_buf[8..128]).expect("Found invalid UTF-8");
                name = name.trim_matches(char::from(0));
            }
            let curr_inode = Inode::new(dc.inode.fs, 0, inum);
            curr_inode.read_inode();
            let curr_file = match curr_inode.dirtype {
                DIR_TYPE => Directory(DirectoryContent{
                    entries: None,
                    inode: curr_inode,
                }),
                FILE_TYPE => DataFile(curr_inode),
            };
            entry_map.insert(&name, curr_file);
        }
        dc.entries = Some(entry_map);
    }

    // write to disk
    fn write_dir(&self) {
        let dc = match self{
             &Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        let entry_map = match dc.entries {
            None => HashMap::new(),
            Some(hm) => hm,
        };
        let iters = entry_map.len();
        let mut write_buf:Vec<u8> = vec![0; iters * 128];
        let mut start = 0;
        for (name, curr_file) in &entry_map { 
            let curr_inum = match curr_file {
                DataFile(inode) => inode.inum,
                Directory(dirc) => dirc.inode.inum,
                _ => panic!("empty file!"),
            };
            let mut this_entry = &mut write_buf[start..(start+128)];
            unsafe{
                let tmp = mem::transmute::<usize, [u8; 8]>(curr_inum);
                this_entry[0..8].copy_from_slice(&tmp[0..8]);
                let source = name.as_bytes();
                for i in 0..120 {
                    if i < source.len() { 
                        this_entry[i+8] = source[i];
                    }else{
                        this_entry[i+8] = 0;
                    }
                }
            }
        }
        dc.inode.write(0, &write_buf);
    }
}
