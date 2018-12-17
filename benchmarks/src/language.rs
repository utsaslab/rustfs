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

fn usage() {

}

/// strip surround quotes for the given string
fn strip(s: String) -> String {
    let mut t = s.clone();
    t.remove(0);
    t.remove(t.len()-1);
    t
}

/// Use `dd` to benchmark the throughput for sequential write
fn dd_seq(dict: &toml::Value) {
    let mut of_path = strip(dict["common"]["SSD_PATH"].to_string());
    let of_path = [of_path, String::from("testfile")].join("/");
    let mut bs = strip(dict["sequential_write"]["BS"].to_string());
    let count = dict["sequential_write"]["COUNT"].to_string();
    let oflag = strip(dict["sequential_write"]["oflag"].to_string());

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

/// Use the SPDK framework to perform sequential write
/// and calculate the throughput
fn rust_seq(dict: &toml::Value) {
    
}

// parse the configuration file
fn parse_config() -> Result<toml::Value> {
    let contents = fs::read_to_string("config/language.toml")
        .expect("Something went wrong reading the file");

    let value = contents.parse::<Value>().unwrap();

    Ok(value)
}

pub fn main() {
    let dict = parse_config().unwrap();
    dd_seq(&dict);
    rust_seq(&dict);
}