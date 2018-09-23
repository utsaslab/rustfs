This directory contains Rust version of hello_nvme_bdev.c program. It is used as a demo
of how we can invoke SPDK library using Rust.

## Usage

Pointing `SPDK_INSTALL_DIR` to the installation location of SPDK and run the following commands:

```
$ export SPDK_INSTALL_DIR=$HOME/spdk_install
$ export RUSTFLAGS="-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"
$ sudo -E $HOME/.cargo/bin/cargo run
```

More details see [Pull Request #14](https://github.com/utsaslab/rustfs/pull/14)
