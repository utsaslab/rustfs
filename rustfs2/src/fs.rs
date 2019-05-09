//! file system interface

use crate::Device;
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
use failure::Error;


use crate::constants::{DEFAULT_SERVER1_SOCKET_PATH, DEFAULT_SERVER2_SOCKET_PATH, FS_SHUTDOWN, FS_OPEN};

#[derive(PartialEq, Debug, Clone, Copy)]
enum FS_OPS {
    NO_OP,
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
        message {
            ops: FS_SHUTDOWN,
        }
    }

    fn open_msg() -> message {
        message {
            ops: FS_OPEN,
        }
    }
}


/// Handle the request from application
fn handle_client(stream: UnixStream) -> FS_OPS {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap().as_str() {
            FS_SHUTDOWN => return FS_OPS::SHUTDOWN,
            FS_OPEN => {
                FS_OPS::OPEN
            }
            _ => FS_OPS::UNSUPPORTED
        };
    }
    FS_OPS::NO_OP
}

fn start_server1() {
    let listener = UnixListener::bind(DEFAULT_SERVER1_SOCKET_PATH).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = thread::spawn(|| handle_client(stream));
                let res = handle.join().unwrap();
                dbg!(res);
                if res == FS_OPS::SHUTDOWN {
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
fn handle_server1(stream: UnixStream) -> FS_OPS {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        match line.unwrap().as_str() {
            FS_SHUTDOWN => return FS_OPS::SHUTDOWN,
            FS_OPEN => {
                FS_OPS::OPEN
            },
            _ => FS_OPS::UNSUPPORTED,
        };       
    }
    FS_OPS::NO_OP
}

/// Open()
async fn open() -> Result<(), Error> {
    unimplemented!();
}

#[allow(unused_variables)]
async fn start_server2(poller: spdk_rs::io_channel::PollerHandle) {
    
    let device = Device::new();
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
                /// at least a async call to open can work but it is not right because we can now only handle one client at a time.
                /// This might cause problem as if the application directly talks to server2, then there is only one socket available
                /// and thus, the request first connected gets advantage: we always wait for its requests until it disconnected.
                /// This limitation majorly due to our SPDK async has to poll on the current thread (i.e., cannot poll in different thread;
                /// if so, we would use tokio). As for now, one possible solution is to use multiple sockets in non-blocking fashion and terminates socket connection on close
                /// If we use two servers architecture, then server1 can help with synchronize (e.g., open on the same file will be queued)
                let stream = BufReader::new(stream);
                for line in stream.lines() {
                    match line.unwrap().as_str() {
                        FS_OPEN => {
                            await!(open());
                        },
                        _ => {}
                    };
                }
                // let handle = thread::spawn(|| handle_server1(stream));
                // let res = handle.join().unwrap();
                // dbg!(res);
                // if res == FS_OPS::SHUTDOWN {
                //     break;
                // }
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

    /// Open()
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
