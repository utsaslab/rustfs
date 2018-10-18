#![feature(nll)]

extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;

mod event;
mod bdev;
mod context;
mod env;
mod bdev_module;

pub use event::{SpdkAppOpts, spdk_app_stop};
pub use bdev::{SpdkBdev, SpdkBdevDesc, SpdkIoChannel};
pub use context::{AppContext};
pub use env::{Buf};
pub use bdev_module::{SpdkBdevIO};
