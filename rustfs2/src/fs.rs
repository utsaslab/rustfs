//! file system interface
use crate::bitmap::Bitmap;
use crate::constants::INODE_SIZE;
use crate::device::Device;
use crate::file::{DirectoryContent, File, File::Directory};
use failure::Error;
use nix::sys::signal::*;
use nix::unistd::*;
use std::fs;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::mem;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process;
use std::thread;
use std::rc::Rc;

use crate::constants::{
    DEFAULT_SERVER1_SOCKET_PATH, DEFAULT_SERVER2_SOCKET_PATH, FS_MKFS, FS_OPEN, FS_SHUTDOWN,
};

pub static mut fs_internal: Option<FsInternal> = None;

#[derive(PartialEq, Debug, Clone, Copy)]
enum FsOps {
    NOOP,
    SHUTDOWN,
    OPEN,
    UNSUPPORTED,
}

/// message that is used to pass between
/// server and client, server and server
/// it contains all the necessary fields
/// supported by FS calls
struct message {
    ops: &'static str,
}

impl message {
    fn shutdown_msg() -> message {
        message { ops: FS_SHUTDOWN }
    }

    fn open_msg() -> message {
        message { ops: FS_OPEN }
    }
}

/// Handle the request from application
fn handle_client(stream: UnixStream) -> FsOps {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap().as_str() {
            FS_SHUTDOWN => return FsOps::SHUTDOWN,
            FS_OPEN => FsOps::OPEN,
            _ => FsOps::UNSUPPORTED,
        };
    }
    FsOps::NOOP
}

/// Start server1 and handle the connection based on operations
fn start_server1() {
    let listener = UnixListener::bind(DEFAULT_SERVER1_SOCKET_PATH).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = thread::spawn(|| handle_client(stream));
                let res = handle.join().unwrap();
                dbg!(res);
                if res == FsOps::SHUTDOWN {
                    break;
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}

/// Handle the request from server1
fn handle_server1(stream: UnixStream) -> FsOps {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap().as_str() {
            FS_SHUTDOWN => return FsOps::SHUTDOWN,
            FS_OPEN => FsOps::OPEN,
            _ => FsOps::UNSUPPORTED,
        };
    }
    FsOps::NOOP
}

#[allow(unused_variables)]
async fn start_server2(poller: spdk_rs::io_channel::PollerHandle) {
    let listener = match UnixListener::bind(DEFAULT_SERVER2_SOCKET_PATH) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {:?}", e);
            return;
        }
    };
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                // At least a async call to open can work but it is not right because we can now only handle one client at a time.
                // This might cause problem as if the application directly talks to server2, then there is only one socket available
                // and thus, the request first connected gets advantage: we always wait for its requests until it disconnected.
                // This limitation majorly due to our SPDK async has to poll on the current thread (i.e., cannot poll in different thread;
                // if so, we would use tokio). As for now, one possible solution is to use multiple sockets in non-blocking fashion and terminates socket connection on close
                // If we use two servers architecture, then server1 can help with synchronize (e.g., open on the same file will be queued)
                let stream = BufReader::new(stream);
                for line in stream.lines() {
                    match line.unwrap().as_str() {
                        FS_OPEN => match await!(FsInternal::open()) {
                            Ok(_) => {}
                            Err(error) => panic!("{:?}", error),
                        }
                        FS_SHUTDOWN => break,
                        FS_MKFS => match await!(FsInternal::mkfs()) {
                            Ok(_) => {}
                            Err(error) => panic!("{:?}", error)
                        },   
                        _ => {}
                    };
                }
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
    spdk_rs::event::app_stop(true);
}

/// Start SPDK framework
fn start_spdk<
    G: std::future::Future<Output = ()> + 'static,
    F: Fn(spdk_rs::io_channel::PollerHandle) -> G,
>(
    async_fn: F,
) {
    let config_file = Path::new("config/bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("rustfs2");
    opts.config_file(config_file.to_str().unwrap());

    let _ret = opts.start(|| {
        let executor = spdk_rs::executor::initialize();
        mem::forget(executor);

        let poller = spdk_rs::io_channel::poller_register(spdk_rs::executor::pure_poll);
        spdk_rs::executor::spawn(async_fn(poller));
    });

    println!("Successfully shutdown SPDK framework");
}

// pub struct FsInternal<'r> {
//     pub device: Device,
//     //    data_bitmap_base: usize,
//     //    inode_bitmap_base: usize,
//     pub inode_base: usize,
//     pub data_base: usize,
//     pub inode_bitmap: Bitmap,
//     pub data_bitmap: Bitmap,
//     //pub root: Option<File<'r>>,
// }

/// Internal FS state data structure
pub struct FsInternal {
    pub device: Device,
    //    data_bitmap_base: usize,
    //    inode_bitmap_base: usize,
    pub inode_base: usize,
    pub data_base: usize,
    pub inode_bitmap: Bitmap,
    pub data_bitmap: Bitmap,
    //pub root: Option<File<'r>>,    
}

impl FsInternal {    
    /// Open()
    async fn open() -> Result<(), Error> {
        unimplemented!();
    }

    /// mkfs()
    async fn mkfs() -> Result<(), Error> {
        let device = Device::new();
        let blk_size = device.blk_size();
        unsafe {
            fs_internal = Some(FsInternal {
                device: device,
                inode_base: 3 * blk_size,
                data_base: 3 * blk_size + INODE_SIZE * blk_size * 8,
                inode_bitmap: Bitmap::new(blk_size, blk_size),
                data_bitmap: Bitmap::new(2 * blk_size, blk_size),
                //root: None,
            });
        }
        // Let's persistent the FS structure onto disk
        let zero_buf = spdk_rs::env::dma_zmalloc(device.blk_size(), device.buf_align());
        let mut write_buf = spdk_rs::env::dma_zmalloc(device.blk_size(), device.buf_align());
        write_buf.fill(device.blk_size(), "%s", "RustFS--");
        await!(device.write(&write_buf, 0, device.blk_size()))?;

        // Define - root lives in first inode
        let byte: [u8; 1] = [1; 1];
        write_buf.fill_bytes(&byte[..]);
        await!(device.write(&write_buf, device.blk_size(), device.blk_size()))?;
        await!(device.write(&zero_buf, 2 * device.blk_size(), device.blk_size()))?;
        //let root_inode = inode::Inode::new(&mut self, DIR_TYPE, 0);
        //root_inode.get_or_alloc_page(0);
        //root_inode.write_inode();
        //self.make_root(root_inode);
        Ok(())
    }

    pub async fn alloc_block() -> usize {
        let fs = fs_internal.unwrap();
        let index = fs.data_bitmap.find().unwrap();
        let offset = index * fs.device.blk_size() + fs.data_base;
        let zero_buf = spdk_rs::env::dma_zmalloc(fs.device.blk_size(), fs.device.buf_align());
        await!(fs.device.write(&zero_buf, offset, fs.device.blk_size()));
        offset / fs.device.blk_size()
    }


    async fn mount() -> Result<(), Error> {
        let fs = fs_internal.unwrap();
        let mut read_buf = spdk_rs::env::dma_zmalloc(fs.device.blk_size(), fs.device.buf_align());
        await!(fs.device.read(
            &mut read_buf,
            fs.device.blk_size(),
            fs.device.blk_size(),
        ));
        fs.inode_bitmap
            .bitmap
            .copy_from_slice(read_buf.read_bytes(fs.device.blk_size()));
        await!(fs.device.read(
            &mut read_buf,
            2 * fs.device.blk_size(),
            fs.device.blk_size(),
        ));
        fs.data_bitmap
            .bitmap
            .copy_from_slice(read_buf.read_bytes(fs.device.blk_size()));
        await!(fs.device.read(
            &mut read_buf,
            3 * fs.device.blk_size(),
            fs.device.blk_size(),
        ));
        // let root_inode: Inode;
        // root_inode.read_inode();
        // self.make_root(root_inode);
        Ok(())
    }

    // fn make_root(&mut self, root_inode: Inode<'r>) {
    //     let dir_content = DirectoryContent {
    //         entries: None,
    //         inode: root_inode,
    //     };
    //     self.root = Some(Directory(dir_content));
    // }
}

/// Public API for the user
pub struct FS {}

impl FS {
    /// We create a new file system instance
    /// There are two servers on two processes we need to start:
    /// server1: a server to take request from application
    /// server2: a server that runs SPDK framework
    /// Whenever there is a request from application, it is sent to server1 and server1 will
    /// initiate a RPC request to server2 and server2 will perform actual heavlifting work and return result
    /// back to server1, which will return result back to client application.
    pub fn new() -> std::io::Result<()> {
        if Path::new(DEFAULT_SERVER1_SOCKET_PATH).exists() {
            fs::remove_file(DEFAULT_SERVER1_SOCKET_PATH)?;
        }
        if Path::new(DEFAULT_SERVER2_SOCKET_PATH).exists() {
            fs::remove_file(DEFAULT_SERVER2_SOCKET_PATH)?;
        }
        match fork().expect("fork failed") {
            // We're in the parent process; we start server1
            ForkResult::Parent { child } => {
                start_server1();
                Ok(())
            }
            ForkResult::Child => {
                // We're in the child process; we start server2
                start_spdk(start_server2);
                Ok(())
            }
        }
    }

    /// open()
    pub fn open() -> usize {
        let mut stream = UnixStream::connect(DEFAULT_SERVER1_SOCKET_PATH).unwrap();
        stream.write_all(FS_OPEN.as_bytes()).unwrap();
        0
    }

    /// Shutdown FS
    pub fn shutdown() {
        let msg = message::shutdown_msg();
        let mut stream = UnixStream::connect(DEFAULT_SERVER2_SOCKET_PATH).unwrap();
        stream.write_all(FS_SHUTDOWN.as_bytes()).unwrap();
        stream = UnixStream::connect(DEFAULT_SERVER1_SOCKET_PATH).unwrap();
        stream.write_all(FS_SHUTDOWN.as_bytes()).unwrap();
    }
}
