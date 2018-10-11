#![feature(nll)]

extern crate libspdk_sys as raw;
#[macro_use]
extern crate failure;
extern crate libc;

mod event;
mod bdev;
mod context;

pub use event::{SpdkAppOpts, spdk_app_stop};
pub use bdev::{SpdkBdev, SpdkBdevDesc};
pub use context::{AppContext};
