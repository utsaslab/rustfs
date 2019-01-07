//! Contains various utility functions used across crates in the repo
#![feature(duration_float)]
#[macro_use]
extern crate log;
extern crate env_logger;

use std::time::Duration;

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
/// - convert("1", "MB", "KB") -> "1024KB"
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
}
