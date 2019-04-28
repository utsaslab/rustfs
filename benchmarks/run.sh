#!/bin/bash

# All commands should be run with `sudo`

source $HOME/.cargo/env
export RUST_BACKTRACE=1

# Pointing `SPDK_INSTALL_DIR` to the installation location of SPDK and run the following commands:
export SPDK_INSTALL_DIR=$HOME/spdk_install
export RUSTFLAGS="-C link-arg=$SPDK_INSTALL_DIR/lib/libspdk.so"

# Logging level
export RUSTFS_BENCHMARKS_LANGUAGE_LOG=debug

# Whether to use memory (e.g., Malloc0) instead of NVMe disk for the benchmark
export MALLOC0=0

if [ "$1" =  "clean" ]; then
    cargo clean
elif [ "$1" = "test" ]; then
    rm -rf run_inner_check2_test_file_new.txt
    rm -rf run_inner_check2_test_file_origin.txt
    rm -rf checksum_new.txt
    rm -rf checksum_origin.txt
    cargo test -- --nocapture
    #cargo test -- --nocapture
elif [ "$1" == "dd" ]; then
    cargo run 1 0
elif [ "$1" == "seq" ]; then
    cargo run 1 1
elif [ "$1" == "rand" ]; then
    cargo run 0 1
fi
