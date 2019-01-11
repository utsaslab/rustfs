/*************************************************************************
  > File Name:       language.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    12/15/18
  > Description:
    
    Benchmark how much overhead is imposed by Rust language wrapper
 ************************************************************************/

use std::process;
use std::path::Path;
use std::fs;
use toml::Value;
use std::mem;
use failure::Error;
use utils_rustfs;
use std::env;
use env_logger::Builder;
use std::time::{Duration, Instant};

fn usage() {

}

/// Use `dd` to benchmark the throughput for sequential write
fn dd_seq() {

    let dict = parse_config().unwrap();
    
    let mut of_path = utils_rustfs::strip(dict["common"]["SSD_PATH"].to_string());
    let of_path = [of_path, String::from("testfile")].join("/");
    let mut bs = utils_rustfs::strip(dict["sequential_write"]["BS"].to_string());
    let count = dict["sequential_write"]["COUNT"].to_string();
    let oflag = utils_rustfs::strip(dict["sequential_write"]["oflag"].to_string());

    let mut command = "dd ".to_owned();
    command = command + "if=/dev/zero" + " " +
        "of=" + of_path.as_str() + " " +
        "bs=" + bs.as_str() + " " +
        "count=" + count.as_str() + " " +
        "oflag=" + oflag.as_str() + " ";
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
    let v2_string: Vec<&str> = v2[v2.len()-1].split(" ").collect();
    // e.g., 723
    let throughput = v2_string[1];

    println!("Throughput: {:} MB/s", v2_string[1]);
    assert!(output.status.success());
}

async fn run(poller: spdk_rs::io_channel::PollerHandle) {
    match await!(run_inner()) {
        Ok(_) => println!("Successful"),
        Err(err) => println!("Failure: {:?}", err),
    }
    spdk_rs::event::app_stop(true);
}

async fn run_inner() -> Result<(), Error> {

    let dict = parse_config().unwrap();

    let mut of_path = utils_rustfs::strip(dict["common"]["SSD_PATH"].to_string());
    let of_path = [of_path, String::from("testfile")].join("/");
    let mut bs = utils_rustfs::strip(dict["sequential_write"]["BS"].to_string());
    let count = dict["sequential_write"]["COUNT"].to_string();
    let oflag = utils_rustfs::strip(dict["sequential_write"]["oflag"].to_string());

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
    let write_size = utils_rustfs::convert(num_int.to_string().as_str(), unit.as_str(), "B");
    debug!("write_size: {}", write_size);
    let write_size_numeric: u64 = write_size.parse::<u64>().unwrap();
    
    //let ret = spdk_rs::bdev::get_by_name("Malloc0");
    let ret = spdk_rs::bdev::get_by_name("Nvme0n1");    
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

    // we want to prepare a buffer with random content to write
    let mut write_buf = spdk_rs::env::dma_zmalloc(write_size_numeric as usize, buf_align);
    write_buf.fill_random(write_size_numeric as usize);

    debug!("write_size_numeric: {}", write_size_numeric);
    // let's time the execution of the write
    let start = Instant::now();
    // await!( async {
    //     spdk_rs::bdev::write(desc.clone(), &io_channel, &write_buf, 0, write_size_numeric)
    // });
    match await!(spdk_rs::bdev::write(desc.clone(), &io_channel, &write_buf, 0, write_size_numeric)) {
        Ok(_) => println!("Successfully write to bdev"),
        _ => {}
    }
    let duration = start.elapsed();
    println!("Successfully write to bdev");
    
    debug!("Time elapsed in write {} bytes is: {:?}", write_size, duration);
    let throughput = write_size_numeric as f64 / utils_rustfs::convert_time(duration, "s");
    debug!("throughput: {} byte/s", throughput);

    println!("throughput: {} MB/s", utils_rustfs::convert(throughput.to_string().as_str(), "B", "MB"));
    
    spdk_rs::thread::put_io_channel(io_channel);
    spdk_rs::bdev::close(desc);
    spdk_rs::event::app_stop(true);
    Ok(())
}

/// Use the SPDK framework to perform sequential write
/// and calculate the throughput
fn rust_seq() {
    let config_file = Path::new("config/bdev.conf").canonicalize().unwrap();
    let mut opts = spdk_rs::event::SpdkAppOpts::new();

    opts.name("language");
    opts.config_file(config_file.to_str().unwrap());

    let _ret = opts.start(|| {
        let executor = spdk_rs::executor::initialize();
        mem::forget(executor);

        let poller = spdk_rs::io_channel::poller_register(spdk_rs::executor::pure_poll);
        spdk_rs::executor::spawn(run(poller));
    });

    println!("Successfully shutdown SPDK framework");
}

/// parse the configuration file "language.toml"
fn parse_config() -> Result<toml::Value, Error> {
    let contents = fs::read_to_string("config/language.toml")
        .expect("Something went wrong reading the file");

    let value = contents.parse::<Value>().unwrap();

    Ok(value)
}

pub fn main() {
    Builder::new()
        .parse(&env::var("RUSTFS_BENCHMARKS_LANGUAGE_LOG").unwrap_or_default())
        .init();
       
    //dd_seq();
    rust_seq();
}
