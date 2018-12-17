/*************************************************************************
  > File Name:       language.rs
  > Author:          Zeyuan Hu
  > Mail:            iamzeyuanhu@utexas.edu
  > Created Time:    12/15/18
  > Description:
    
    Benchmark how much overhead is imposed by Rust language wrapper
 ************************************************************************/

use std::process;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::io::{BufRead, BufReader, Result};
use std::collections::HashMap;
use std::fs;
use toml::Value;

macro_rules! build_from_paths {
    ($base:expr, $($segment:expr),+) => {{
        let mut base: ::std::path::PathBuf = $base.into();
        $(
            base.push($segment);
        )*
        base
    }}
}

fn usage() {

}

/// Use `dd` to benchmark the throughput and latency for sequential write
fn dd_seq(dict: &HashMap<String, String>) {
    let of_path = build_from_paths!(dict.get("SSD_PATH").unwrap(), "testfile");
    let bs = dict.get("BS").unwrap();
    let count = dict.get("COUNT").unwrap();

    let output = process::Command::new("dd")
        .arg("if=/dev/zero")
        .arg(["of=", of_path.to_str().unwrap()].join(""))
        .arg(["bs=", bs].join(""))
        .arg(["count=", count].join(""))
        .arg("oflag=direct")
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

/// Use native Rust to benchmark
fn test_rust() {

}

// parse the configuration file
fn parse_config() -> Result<HashMap<String,String>> {
    let mut dict = HashMap::new();

    let path = Path::new("config/language.conf");
    let display = path.display();

    let file = match File::open(&path) {
        Err(why) => panic!("Couldn't open {}: {}", display, why.description()),
        Ok(file) => file
    };

    let contents = fs::read_to_string(filename)
        .expect("Something went wrong reading the file");
    println!("{:}", contents);
    Ok(dict)
}

pub fn main() {
    let dict = parse_config().unwrap();
    dd_seq(&dict);
}