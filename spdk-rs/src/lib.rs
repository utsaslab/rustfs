#![feature(nll)]
#![feature(pin)]
#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(tool_lints)]
#![allow(macro_use_extern_crate)]
#![feature(uniform_paths)]
#![feature(arbitrary_self_types)]

extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;
#[cfg(test)]
#[macro_use]
extern crate hamcrest2;
extern crate futures;
extern crate futures_new;
#[macro_use]
extern crate tokio;

#[macro_use]
extern crate log;
extern crate env_logger;
extern crate utils_rustfs;

pub mod bdev;
pub mod bdev_module;
pub mod context;
pub mod env;
pub mod event;
pub mod executor;
pub mod io_channel;
pub mod run;
pub mod thread;

pub use bdev::{SpdkBdev, SpdkBdevDesc};
pub use bdev_module::SpdkBdevIO;
pub use context::{AppContext, SpdkBdevIoCompletionCb};
pub use env::Buf;
pub use event::{app_stop, SpdkAppOpts};
