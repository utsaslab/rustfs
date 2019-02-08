/*************************************************************************
 > File Name:       language.rs
 > Author:          Zeyuan Hu
 > Mail:            iamzeyuanhu@utexas.edu
 > Created Time:    12/15/18
 > Description:

   Benchmark:

   1. Sequential write throughput for Rust + SPDK vs. dd
   2. Random write latency for Rust + SPDK vs. dd
************************************************************************/

use colored::*;
use env_logger::Builder;
use failure::Error;
use futures;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::mem;
use std::path::Path;
use std::process;
use std::thread;
use std::time::{Duration, Instant};
use toml::Value;
use utils_rustfs;

fn usage() {}

/// Use `dd` to benchmark the throughput for sequential write
fn dd_seq() {
    let dict = parse_config().unwrap();

    let mut of_path = utils_rustfs::strip(dict["common"]["SSD_PATH"].to_string());
    let of_path = [of_path, String::from("testfile")].join("/");
    let mut bs = utils_rustfs::strip(dict["sequential_write"]["BS"].to_string());
    let count = dict["sequential_write"]["COUNT"].to_string();
    let oflag = utils_rustfs::strip(dict["sequential_write"]["oflag"].to_string());

    let mut command = "dd ".to_owned();
    command = command
        + "if=/dev/zero"
        + " "
        + "of="
        + of_path.as_str()
        + " "
        + "bs="
        + bs.as_str()
        + " "
        + "count="
        + count.as_str()
        + " "
        + "oflag="
        + oflag.as_str()
        + " ";
    println!("Command: {}", command);

    let output = process::Command::new("dd")
        .arg("if=/dev/zero")
        .arg(["of=", of_path.as_str()].join(""))
        .arg(["bs=", bs.as_str()].join(""))
        .arg(["count=", count.as_str()].join(""))
        .arg(["oflag=", oflag.as_str()].join(""))
        .output()
        .expect("failed to execute process");

    let output_string = String::from_utf8_lossy(&output.stderr).into_owned();
    let v: Vec<&str> = output_string.split("\n").collect();
    // e.g., 1073741824 bytes (1.1 GB, 1.0 GiB) copied, 1.48467 s, 723 MB/s
    let v_string = v[2].to_string();
    // e.g., 1073741824 bytes (1.1 GB, 1.0 GiB) copied, 1.48467 s, 723 MB/s split
    let v2: Vec<&str> = v_string.split(",").collect();
    // e.g.,  723 MB/s split
    let v2_string: Vec<&str> = v2[v2.len() - 1].split(" ").collect();
    // e.g., 723
    let throughput = v2_string[1];

    println!("Throughput: {:} MB/s", v2_string[1]);
    assert!(output.status.success());
}

async fn run(poller: spdk_rs::io_channel::PollerHandle, _test_path_enabled: bool) {
    match await!(run_inner(_test_path_enabled)) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err),
    }
    if _test_path_enabled {
        match await!(run_inner_check2()) {
            Ok(_) => println!("Successful"),
            Err(err) => println!("Failure: {:?}", err),
        }
    }
    spdk_rs::event::app_stop(true);
}

async fn run_inner(_test_path_enabled: bool) -> Result<(), Error> {
    let dict = parse_config().unwrap();

    let mut bs = String::new();
    let mut count = String::new();

    if _test_path_enabled {
        bs = utils_rustfs::strip(dict["sequential_write_test"]["BS"].to_string());
        count = String::from("1");
    } else {
        bs = utils_rustfs::strip(dict["sequential_write"]["BS"].to_string());
        count = dict["sequential_write"]["COUNT"].to_string();
    }

    // let's first calculate how much we should write to the device
    let mut num = String::from("");
    let mut unit = String::from("");
    for c in bs.chars() {
        if c.is_alphabetic() {
            unit.push(c);
        } else {
            num.push(c);
        }
    }

    debug!("num: {}", num);
    debug!("unit: {}", unit);
    debug!("count: {}", count);

    let mut num_int = num.parse::<u64>().unwrap();
    let count_int = count.parse::<u64>().unwrap();
    num_int = num_int * count_int;
    let bs = utils_rustfs::convert(num_int.to_string().as_str(), unit.as_str(), "B");
    debug!("bs: {}", bs);

    let ret = spdk_rs::bdev::get_by_name("Malloc0");
    //let ret = spdk_rs::bdev::get_by_name("Nvme0n1");
    let bdev = ret.unwrap();
    let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();

    // check whether device has volatile write cache enabled
    // if it's true, we may want to call `spdk_bdev_flush()` to flush the writes (not implemented for now)
    let is_write_cache_enabled = spdk_rs::bdev::has_write_cache(bdev.clone());
    debug!("is_write_cache_enabled: {}", is_write_cache_enabled);

    match spdk_rs::bdev::open(bdev.clone(), true, &mut desc) {
        Ok(_) => println!("Successfully open the device {}", bdev.name()),
        _ => {}
    };

    let io_channel = spdk_rs::bdev::get_io_channel(desc.clone())?;

    let blk_size = spdk_rs::bdev::get_block_size(bdev.clone());
    println!("blk_size: {}", blk_size);

    let buf_align = spdk_rs::bdev::get_buf_align(bdev.clone());
    println!("buf_align: {}", buf_align);

    // We need to round `bs` to be the multiple of `blk_size`
    let bs_numeric: f64 = bs.parse::<f64>().unwrap();
    let num_blks = (bs_numeric / blk_size as f64).ceil();
    let write_size_numeric: u64 = (blk_size as f64 * num_blks) as u64;

    // We divide the write_size_numeric into 1MB chunk (see issue: https://github.com/spdk/spdk/issues/578)
    let write_buf_size: usize = utils_rustfs::constant::MEGABYTE;
    let num_chunks = (write_size_numeric as f64 / write_buf_size as f64).floor() as usize;

    // we want to prepare a vector of buffers with random content
    let mut buffer_vec = Vec::new();
    for i in 0..num_chunks {
        let mut write_buf = spdk_rs::env::dma_zmalloc(write_buf_size, buf_align);
        write_buf.fill_fixed(write_buf_size, "A");
        buffer_vec.push(write_buf);
    }

    debug!("write_size_numeric: {}", write_size_numeric);
    debug!("num_chunks: {}", num_chunks);
    // let's time the execution of the write
    let start = Instant::now();
    for i in 0..num_chunks {
        match await!(spdk_rs::bdev::write(
            desc.clone(),
            &io_channel,
            &buffer_vec[i],
            (i * write_buf_size) as u64,
            write_buf_size as u64
        )) {
            Ok(_) => {}
            Err(error) => panic!("{:?}", error),
        }
    }
    let duration = start.elapsed();

    debug!(
        "Time elapsed in write {} bytes is: {:?}",
        write_size_numeric, duration
    );
    let throughput = write_size_numeric as f64 / utils_rustfs::convert_time(duration, "s");
    debug!("throughput: {} byte/s", throughput);

    println!(
        "throughput: {} MB/s",
        utils_rustfs::convert(throughput.to_string().as_str(), "B", "MB")
    );

    if _test_path_enabled {
        print!("{}", "Correctness check 1".green());
        let mut read_buf = spdk_rs::env::dma_zmalloc(write_buf_size, buf_align);
        for i in 0..num_chunks {
            match await!(spdk_rs::bdev::read(
                desc.clone(),
                &io_channel,
                &mut read_buf,
                (i * write_buf_size) as u64,
                write_buf_size as u64
            )) {
                Ok(_) => {
                    // We check the buffer
                    unsafe {
                        for i in 0..write_buf_size {
                            assert!(
                                *(read_buf.to_raw() as *mut u8).offset(i as isize) as char == 'A'
                            );
                        }
                    }
                }
                Err(error) => panic!("{:}", error),
            }
        }
        println!("{}", " ... ok".green());
    }

    spdk_rs::thread::put_io_channel(io_channel);
    spdk_rs::bdev::close(desc);
    spdk_rs::event::app_stop(true);
    Ok(())
}

/// Use the SPDK framework to perform sequential write
/// and calculate the throughput
fn rust_seq<
    G: std::future::Future<Output = ()> + 'static,
    F: Fn(spdk_rs::io_channel::PollerHandle, bool) -> G,
>(
    async_fn: F,
    _test_path_enabled: bool,
) {
    let config_file = Path::new("config/bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("language");
    opts.config_file(config_file.to_str().unwrap());

    let _ret = opts.start(|| {
        let executor = spdk_rs::executor::initialize();
        mem::forget(executor);

        let poller = spdk_rs::io_channel::poller_register(spdk_rs::executor::pure_poll);
        spdk_rs::executor::spawn(async_fn(poller, _test_path_enabled));
    });

    println!("Successfully shutdown SPDK framework");
}

async fn run_inner_check2() -> Result<(), Error> {
    print!("{}", "Correctness check 2".green());

    // For simplicity, file_size should be multiple of 1MB
    let num_chunks = 1;
    //let write_buf_size = utils_rustfs::constant::MEGABYTE;
    let write_buf_size = utils_rustfs::constant::BYTE * 1024;
    let file_size = write_buf_size * num_chunks;

    // We first generate a large random file
    let filename = "run_inner_check2_test_file_origin.txt";
    utils_rustfs::generate_file_random(filename, file_size);

    // We write the file to the disk using SPDK
    let ret = spdk_rs::bdev::get_by_name("Nvme0n1");
    let bdev = ret.unwrap();
    let mut desc = spdk_rs::bdev::SpdkBdevDesc::new();

    match spdk_rs::bdev::open(bdev.clone(), true, &mut desc) {
        Ok(_) => println!("Successfully open the device {}", bdev.name()),
        _ => {}
    };
    let io_channel = spdk_rs::bdev::get_io_channel(desc.clone())?;
    let blk_size = spdk_rs::bdev::get_block_size(bdev.clone());
    let buf_align = spdk_rs::bdev::get_buf_align(bdev.clone());

    let mut buffer_vec = Vec::new();
    for i in 0..num_chunks {
        let mut write_buf = spdk_rs::env::dma_zmalloc(write_buf_size, buf_align);
        let num_read = write_buf.fill_from_file(filename, i * write_buf_size, write_buf_size);
        debug!("num_read: {}", num_read);
        buffer_vec.push(write_buf);
    }
    utils_rustfs::getLine!();

    for i in 0..num_chunks {
        utils_rustfs::getLine!();
        match await!(spdk_rs::bdev::write(
            desc.clone(),
            &io_channel,
            &buffer_vec[i],
            (i * write_buf_size) as u64,
            write_buf_size as u64
        )) {
            Ok(_) => {}
            Err(error) => panic!("{:?}", error),
        }
    }
    utils_rustfs::getLine!();

    // Let's see part of what we have written
    println!("{}", "Let's see what we have written".yellow());
    let mut read_buf = spdk_rs::env::dma_zmalloc(write_buf_size, buf_align);
    for i in 0..1 {
        match await!(spdk_rs::bdev::read(
            desc.clone(),
            &io_channel,
            &mut read_buf,
            (i * write_buf_size) as u64,
            write_buf_size as u64
        )) {
            Ok(_) => println!("we have written (part of): {}", read_buf.read()),
            Err(error) => panic!("{:}", error),
        }
    }

    // We calculate the check sum of the large random file and save it to the disk.
    // Next time, we check whether such file exists, if so, we read content from disk
    // into another file and compare the checksum.
    // The block can possible be modified between two runs. Thus, we use `run.sh` call
    // `cargo test` twice and ensure there is no others run the same program. In other words, please
    // don't touch SPDK during the test.
    let checksum_filename = "checksum_origin.txt";
    if !Path::new(checksum_filename).exists() {
        debug!("{} not exists!", checksum_filename.yellow());
        utils_rustfs::get_checksum(filename, checksum_filename);
    } else {
        let checksum_filename_new = "checksum_new.txt";
        let filename_new = "run_inner_check2_test_file_new.txt";
        let mut read_buf = spdk_rs::env::dma_zmalloc(write_buf_size, buf_align);
        for i in 0..num_chunks {
            match await!(spdk_rs::bdev::read(
                desc.clone(),
                &io_channel,
                &mut read_buf,
                (i * write_buf_size) as u64,
                write_buf_size as u64
            )) {
                Ok(_) => {
                    // We write the read_buf content into the file
                    let mut file_new = fs::File::create(filename_new)?;
                    file_new = OpenOptions::new().append(true).open(filename_new)?;
                    write!(&mut file_new, "{}", read_buf.read());
                }
                Err(error) => panic!("{:}", error),
            }
        }
        // Let's calculate the new checksum
        utils_rustfs::get_checksum(filename_new, checksum_filename_new)?;
        // Let's compare the checksum
        let mut file = fs::File::open(checksum_filename)?;
        let mut checksum_origin = String::new();
        file.read_to_string(&mut checksum_origin);
        file = fs::File::open(checksum_filename_new)?;
        let mut checksum_new = String::new();
        file.read_to_string(&mut checksum_new);
        assert_eq!(checksum_origin, checksum_new);
        println!("{}", " ... ok".green());
    }

    Ok(())
}

/// Test the Rust + SPDK sequential write correctness
/// We perform the following two checks:
/// 1. Once the write is finished, we immediately read from the disk and see if the read content and write content are exactly the same.
/// 2. a) Generate a big random file b) we write the file to the disk using SPDK c) we reset the driver and shutdown SPDK framework
///    d) We setup the driver and start the SPDK framework again e) we read the content from disk to another file f) we compare sha256 checksums of the two files
#[test]
fn rust_seq_test() {
    Builder::new()
        .parse(&env::var("RUSTFS_BENCHMARKS_LANGUAGE_LOG").unwrap_or_default())
        .init();

    // Perform check 1 and check 2
    rust_seq(run, true);
}

/// parse the configuration file "language.toml"
fn parse_config() -> Result<toml::Value, Error> {
    let contents =
        fs::read_to_string("config/language.toml").expect("Something went wrong reading the file");

    let value = contents.parse::<Value>().unwrap();

    Ok(value)
}

pub fn main() {
    Builder::new()
        .parse(&env::var("RUSTFS_BENCHMARKS_LANGUAGE_LOG").unwrap_or_default())
        .init();

    //dd_seq();
    rust_seq(run, false);
}
