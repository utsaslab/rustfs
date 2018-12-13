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

pub mod event;
pub mod bdev;
pub mod context;
pub mod env;
pub mod bdev_module;
pub mod thread;
pub mod run;

pub use event::{SpdkAppOpts, app_stop};
pub use bdev::{SpdkBdev, SpdkBdevDesc, SpdkIoChannel};
pub use context::{AppContext, SpdkBdevIoCompletionCb};
pub use env::{Buf};
pub use bdev_module::{SpdkBdevIO};
