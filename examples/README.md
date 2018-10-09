This directory contains example programs utilize the [SPDK](http://www.spdk.io/doc/about.html).

- `hello_nvme_bdev`

    An example program that performs a write and a read to the underlying NVMe SSD.
    
- `hello_nvme_bdev_rust`

    Same functionality as `hello_nvme_bdev` but written in Rust with the raw rust bindings.
    
- `hello_nvme_bdev_rust_wrapper`

    Same functionality as `hello_nvme_bdev` but written in Rust the the rust-friendly FFI from
    crate [spdk-rs](https://github.com/utsaslab/rustfs/tree/master/spdk-rs).