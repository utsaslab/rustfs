#![feature(nll)]
#![feature(tool_lints)]
#![warn(rust_2018_idioms)]
#![feature(async_await, await_macro, futures_api)]
#![feature(use_extern_macros)]

extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;
extern crate futures;

mod event;
mod bdev;
mod context;
mod env;
mod bdev_module;

pub use crate::event::{SpdkAppOpts, spdk_app_stop};
pub use crate::bdev::{SpdkBdev, SpdkBdevDesc, SpdkIoChannel};
pub use crate::context::{AppContext, SpdkBdevIoCompletionCb};
pub use crate::env::{Buf};
pub use crate::bdev_module::{SpdkBdevIO};