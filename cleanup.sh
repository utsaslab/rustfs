#!/bin/bash

# Cleanup all the rust modules output
# Run the script with `sudo`

CARGO="$HOME/.cargo/bin/cargo"
cd benchmarks && $CARGO clean && cd -
cd examples/hello_nvme_bdev_rust && $CARGO clean && cd -
cd examples/hello_nvme_bdev_rust_wrapper && $CARGO clean && cd -
cd spdk-rs && $CARGO clean && cd -
cd utils && $CARGO clean && cd -
