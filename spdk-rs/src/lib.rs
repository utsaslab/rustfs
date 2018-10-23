#![feature(nll)]
#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(use_extern_macros)]
#![feature(proc_macro, generators)]


extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate futures_await as futures;
//extern crate futures;

mod event;
mod bdev;
mod context;
mod env;
mod bdev_module;

pub use event::{SpdkAppOpts, spdk_app_stop};
pub use bdev::{SpdkBdev, SpdkBdevDesc, SpdkIoChannel};
pub use context::{AppContext, SpdkBdevIoCompletionCb};
pub use env::{Buf};
pub use bdev_module::{SpdkBdevIO};