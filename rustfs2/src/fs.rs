//! file system interface
use std::io::{BufRead, BufReader};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process;
use std::thread;

use nix::sys::signal::*;
use nix::unistd::*;

use crate::constants::{DEFAULT_SERVER1_SOCKET_PATH, DEFAULT_SERVER2_SOCKET_PATH};

fn handle_client(stream: UnixStream) {
    let stream = BufReader::new(stream);
    for line in stream.lines() {
        println!("{}", line.unwrap());
    }
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
    pub fn new() {
        match fork().expect("fork failed") {
            ForkResult::Parent { child } => {
                // We're in the parent process; we start server1
                let listener = UnixListener::bind(DEFAULT_SERVER1_SOCKET_PATH).unwrap();
                for stream in listener.incoming() {
                    match stream {
                        Ok(stream) => {
                            thread::spawn(|| handle_client(stream));
                        }
                        Err(err) => {
                            println!("Error: {}", err);
                            break;
                        }
                    }
                }
            }
            ForkResult::Child => {
                // We're in the child process; we start server2
                process::exit(0);
            }
        }
    }
}
