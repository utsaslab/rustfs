/*************************************************************************
  > File Name:       run.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    12/13/18
  > Description:
    
    The necessary infrastructure to use SPDK framework from Rust
 ************************************************************************/

use std::future::{Future as StdFuture};
use tokio::runtime::Runtime;
use tokio::prelude::*;
use crate::event;

async fn map_ok<T: StdFuture>(future: T) -> Result<(),()> {
    let _ = await!(future);
    Ok(())
}

pub fn run_spdk<F>(future: F)
    where F: StdFuture<Output = ()> + Send + 'static,
{
    use tokio_async_await::compat::backward;
    let future = backward::Compat::new(map_ok(future));

    let mut rt = Runtime::new().unwrap();
    rt.block_on(future);
    rt.shutdown_now().wait().unwrap();
    event::app_stop(true);
}