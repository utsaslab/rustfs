/*************************************************************************
  > File Name:       wrapper.h
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    8/29/18
  > Description:
    
    This header contains the spdk header files that we want to generate
    the rust bindings from.

    Please make sure `cargo test` can pass (it will verify the layout,
    alighnment, size of generated Rust Foreign Fuction Interface (FFI) structs
    match what bindgen thinks they should be). If something goes wrong,
    you may want to adjust using `blacklist_type` or `opaque_type` inside build.rs
 ************************************************************************/

#ifndef RUSTFS_WRAPPER_H
#define RUSTFS_WRAPPER_H

#include "spdk/stdinc.h"
#include "spdk/thread.h"
#include "spdk/bdev.h"
#include "spdk/env.h"
#include "spdk/event.h"
#include "spdk/log.h"
#include "spdk/string.h"
#include "spdk/bdev_module.h"


#endif //RUSTFS_WRAPPER_H
