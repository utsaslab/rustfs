#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    // We want to be able to access Rust FFI structs inside
    // bindings.rs
    use super::*;
    #[test]
    fn hello_nvme_bdev() {
        unsafe {
            let g_bdev_name = "Nvme0n1";

            struct hello_context_t {
                bdev : spdk_bdev,
                bdev_desc : spdk_bdev_desc,
                bdev_io_channel : spdk_io_channel,
                buff : String,
                bdev_name : String
            }
        }



    }
}

