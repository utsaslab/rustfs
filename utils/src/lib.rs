//! Contains various utility functions used across crates in the repo
#![feature(duration_float)]
#![feature(core_intrinsics)]

#[macro_use]
extern crate log;
extern crate colored;
extern crate env_logger;
extern crate failure;
extern crate rand;

use colored::*;
use failure::Error;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::fs;
use std::io::prelude::*;
use std::path::Path;
use std::process;
use std::time::Duration;

pub mod constant;

/// NOTE: caller needs to have env_loger::init() in its environment to make macro work (i.e., print when log is enabled)
#[macro_export]
macro_rules! getLine {
    ($($msg : expr)*) => {
        debug!("Execution hit line: {}", line!());
    };
}

/// strip surround quotes for the given string
pub fn strip(s: String) -> String {
    let mut t = s.clone();
    t.remove(0);
    t.remove(t.len() - 1);
    t
}

/// convert given string (e.g., "1") with unit `unit1` into corresponding size specified
/// by `unit2`. Function uses `unit1` as the base unit and perform convert.
/// Given string should only be literal.
/// ## Example:
/// - convert("1", "MB", "KB") -> "1024"
/// ## Supported conversion: MB, KB, G, B
pub fn convert(s: &str, _unit1: &str, _unit2: &str) -> String {
    let res: f64;

    match s.parse::<f64>() {
        Ok(t) => {
            if _unit1 == "" || _unit2 == "" {
                panic!("_unit1, _unit2 should not be empty string");
            }
            if _unit2 == "MB" {
                if _unit1 == "KB" {
                    res = t / 1024.0;
                } else if _unit1 == "G" {
                    res = t * 1024.0;
                } else if _unit1 == "B" {
                    res = t / 1024.0 / 1024.0;
                } else {
                    panic!("Unsupported conversion unit");
                }
            } else if _unit2 == "KB" {
                if _unit1 == "MB" {
                    res = t * 1024.0;
                } else if _unit1 == "G" {
                    res = t * 1024.0 * 1024.0;
                } else if _unit1 == "B" {
                    res = t / 1024.0;
                } else {
                    panic!("Unsupported conversion unit");
                }
            } else if _unit2 == "G" {
                if _unit1 == "MB" {
                    res = t / 1024.0;
                } else if _unit1 == "KB" {
                    res = t / 1024.0 / 1024.0;
                } else if _unit1 == "B" {
                    res = t / 1024.0 / 1024.0 / 1024.0;
                } else {
                    panic!("Unsupported conversion unit");
                }
            } else if _unit2 == "B" {
                if _unit1 == "KB" {
                    res = t * 1024.0;
                } else if _unit1 == "MB" {
                    res = t * 1024.0 * 1024.0;
                } else if _unit1 == "G" {
                    res = t * 1024.0 * 1024.0 * 1024.0;
                } else {
                    panic!("Unsupported conversion unit");
                }
            } else {
                panic!("Unsupported conversion unit");
            }
        }
        Err(_e) => panic!("s cannot contains character!"),
    }

    if res.fract() == 0.0 {
        return res.to_string();
    } else {
        format!("{:.9}", res)
    }
}

/// Convert given std::time::Duration into f64 with unit specified by `unit`
/// Support _unit string: "s", "ms", "us", "ns"
pub fn convert_time(duration: Duration, _unit: &str) -> f64 {
    let _seconds = duration.as_float_secs();
    if _unit == "s" {
        _seconds
    } else if _unit == "ms" {
        _seconds * 1_000.0
    } else if _unit == "us" {
        _seconds * 1_000_000.0
    } else if _unit == "ns" {
        _seconds * 1_000_000_000.0
    } else {
        panic!("Unsupport _unit type!");
    }
}

/// Generate `size` byte random string
pub fn generate_string(size: usize) -> Result<String, Error> {
    let mut f = fs::File::open("/dev/urandom")?;
    let mut buffer: Vec<u8> = vec![0; size];
    let size = f.read(buffer.as_mut_slice()).unwrap();
    //debug!("read_size: {}", size);
    let string_buffer = unsafe { String::from_utf8_unchecked(buffer) };
    //debug!("string_buffer: {}", string_buffer);
    //debug!("string_buffer len: {}", string_buffer.len());
    let mut string_buffer_raw = String::from(string_buffer.to_owned());
    string_buffer_raw.truncate(size);
    assert_eq!(size, string_buffer_raw.len());
    Ok(string_buffer_raw)
}

/// Generate `size` bytes file with random content
///  `filename` is expected in absolute path
pub fn generate_file_random(filename: &str, size: usize) -> std::io::Result<()> {
    let rand_string: String = thread_rng().sample_iter(&Alphanumeric).take(size).collect();
    let mut output = fs::File::create(filename)?;
    write!(output, "{}", rand_string)?;
    Ok(())
}

/// Get type of given variable
pub fn print_type_of<T>(_: &T) {
    println!("{}", unsafe { std::intrinsics::type_name::<T>() });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip() {
        assert_eq!(strip(r#""bob""#.to_string()), "bob");
    }

    #[test]
    fn test_convert() {
        assert_eq!(convert("1", "G", "KB"), "1048576");
        assert_eq!(convert("1", "G", "MB"), "1024");
        assert_eq!(convert("1", "G", "B"), "1073741824");
        assert_eq!(convert("1", "MB", "KB"), "1024");
        assert_eq!(convert("1", "MB", "G"), "0.000976562");
        assert_eq!(convert("1", "MB", "B"), "1048576");
        assert_eq!(convert("1", "KB", "G"), "0.000000954");
        assert_eq!(convert("1", "KB", "MB"), "0.000976562");
        assert_eq!(convert("1", "KB", "B"), "1024");
        assert_eq!(convert("1024", "B", "KB"), "1");
        assert_eq!(convert("1048576", "B", "MB"), "1");
        assert_eq!(convert("1048576", "B", "G"), "0.000976562");
    }

    #[test]
    #[should_panic(expected = "Unsupported conversion unit")]
    fn test_convert_panic1() {
        convert("1", "PB", "G");
        convert("1", "KB", "");
    }

    #[test]
    #[should_panic(expected = "s cannot contains character!")]
    fn test_convert_panic2() {
        convert("1KB", "G", "XB");
    }

    #[test]
    fn test_convert_time() {
        assert_eq!(
            convert_time(Duration::from_nanos(1_000_000_123), "s"),
            1.000000123
        );
        assert_eq!(convert_time(Duration::from_secs(5), "ms"), 5000.0);
        assert_eq!(convert_time(Duration::from_millis(2569), "ns"), 2.569e+9);
        assert_eq!(
            convert_time(Duration::from_micros(1_000_002), "us"),
            1_000_002.0
        );
    }

    #[test]
    fn test_generate_string() {
        generate_string(10).unwrap();
        generate_string(2073).unwrap();
    }

    #[test]
    fn test_generate_file_random() -> std::io::Result<()> {
        // we test whether we have the file in the designated path and if the size matches expectation
        let tmp_testfile = "/tmp/rustfs_testfile";
        let file_size = 10 * constant::MEGABYTE;
        generate_file_random(tmp_testfile, file_size)?;
        assert_eq!(Path::new(tmp_testfile).exists(), true);
        let metadata = fs::metadata(tmp_testfile)?;
        assert_eq!(file_size, metadata.len() as usize);
        fs::remove_file(tmp_testfile)?;
        Ok(())
    }

    #[test]
    fn test_print_type_of() {
        print_type_of(&32.90); // prints "f64"
        print_type_of(&vec![1, 2, 4]); // prints "std::vec::Vec<i32>"
        print_type_of(&"foo"); // prints "&str"
    }
}
