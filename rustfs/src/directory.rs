use crate::file;

use file::File;
use file::File::Directory;

pub trait DirectoryHandle<'r>: Sized {
    fn is_dir(&self) -> bool;
    fn insert(&mut self, name: &'r str, file: Self);
    fn remove(&mut self, name: &'r str);
    fn get(&self, name: &'r str) -> Option<Self>;
}

impl<'r> DirectoryHandle<'r> for File<'r> {
    fn is_dir(&self) -> bool {
        match self {
            &Directory(_) => true,
            _ => false,
        }
    }

    fn insert(&mut self, name: &'r str, file: File<'r>) {
        //        let rc = self.get_dir_rc();
        // let mut content = rc.borrow_mut();
        // content.entries.insert(name, file);
        unimplemented!();
    }

    fn remove(&mut self, name: &'r str) {
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
             _ => ,
         };
        //        let rc = self.get_dir_rc();
        // let mut content = rc.borrow_mut();
        // content.entries.remove(&name);
        unimplemented!();
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
             _ => ,
         };

                  
//        match dc.entries.unwrap().get(&name) {
//             None => None,
        //             Some(ref file) => Some((*file).clone()), // It's RC
//             Some() =>
//         }
        dc.entries.unwrap().get(&name)
    }

    // read from disk
    fn read_dir(&self){
        let mut dc = match self{
             &Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        let entry_map = match dc.entries {
            None => HashMap::new(),
            Some(hm) => hm,
        };
        let data:Vec<u8> = vec![0; dc.inode.size]; 
        let slice = &mut data[..];
        dc.inode.read(0, &mut slice);
        let iters:usize = dc.inode.size / 128;
        for i in 0..(iters+1) {
            let inum: usize;
            let name: str;
            let start = 128 * iters;
            unsafe{
                inum = mem::transmute::<[u8; 8], usize>(*array_ref![slice, start, 8]);
                name = mem::transmute::<[u8; 120], str>(*array_ref![slice, start+8, 120]);
            }
            let curr_inode = Inode(dc.inode.fs, 0, inum);
            curr_inode.read_inode();
            let curr_file = match curr_inode.dir_type {
                DIR_TYPE => Directory(DirectoryContent{
                    entries: None,
                    inode: curr_inode,
                }),
                FILE_TYPE => Datafile(curr_inode),
            }
            entry_map.insert(name, curr_file);
        }
        *dc.entries = Some(entry_map);
    }

    // write to disk
    fn write_dir(&self) {
        let mut dc = match self{
             &Directory(dir_content) => dir_content,
             _ => panic!("not a dir"),
        };
        let entry_map = match dc.entries {
            None => HashMap::new(),
            Some(hm) => hm,
        };
        let iters = entry_map.len();
        let mut write_buf:Vec<u8> = vec![0, iters * 128];
        let mut start = 0;
        for (name, curr_file) in &entry_map { 
            let curr_inum = match curr_file {
                DataFile(inode) => inode.inum,
                Directory(dirc) => dirc.inode.inum,
            };
            unsafe{
                let tmp = &mut write_buf[start..(start+8)];
                let slice = mem::transmute::<usize, [u8; 8]>(curr_inum);
                tmp.copy_from_slice(&slice[0..8]);
                let tmp = &mut write_buf[(start+8)..(start+128)];
                let slice = mem::transmute::<str, [u8; 8]>(name);
                tmp.copy_from_slice(&slice[0..120]);
            }
        }
        dc.inode.write(0, write_buf);
    }
}
