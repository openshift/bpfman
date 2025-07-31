// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Journal access for integration tests.

#![allow(dead_code)]

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt::Display;
use std::process::Command;

use retry::delay::Fixed;
use serde::Deserialize;
use std::ffi::OsStr;

#[derive(Debug, Copy, Clone)]
pub enum Journal {
    User,
    System,
}

#[derive(Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    Text(String),
    Array(Vec<String>),
    Binary(Vec<u8>),
}

impl FieldValue {
    pub fn as_text(&self) -> Cow<'_, str> {
        match self {
            FieldValue::Text(v) => Cow::Borrowed(v.as_str()),
            FieldValue::Binary(binary) => String::from_utf8_lossy(binary),
            FieldValue::Array(v) => Cow::Borrowed(v.first().map_or("", |s| s.as_str())),
        }
    }
}

impl Display for FieldValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_text().fmt(f)
    }
}

// Convenience impls to compare fields against strings and bytes with assert_eq!
impl PartialEq<&str> for FieldValue {
    fn eq(&self, other: &&str) -> bool {
        match self {
            FieldValue::Text(s) => s == other,
            FieldValue::Binary(b) => b == other.as_bytes(),
            FieldValue::Array(_) => false,
        }
    }
}

// Convenience impls to compare fields against strings and bytes with assert_eq!
impl PartialEq<String> for FieldValue {
    fn eq(&self, other: &String) -> bool {
        self == &other.as_str()
    }
}

// Convenience impls to compare fields against strings and bytes with assert_eq!
impl PartialEq<&String> for FieldValue {
    fn eq(&self, other: &&String) -> bool {
        self == &other.as_str()
    }
}

impl PartialEq<[u8]> for FieldValue {
    fn eq(&self, other: &[u8]) -> bool {
        match self {
            FieldValue::Text(s) => s.as_bytes() == other,
            FieldValue::Binary(data) => data == other,
            FieldValue::Array(_) => false,
        }
    }
}

impl PartialEq<Vec<&str>> for FieldValue {
    fn eq(&self, other: &std::vec::Vec<&str>) -> bool {
        match self {
            FieldValue::Text(_) => false,
            FieldValue::Binary(_) => false,
            FieldValue::Array(a) => a == other,
        }
    }
}

/// Read from journal.
///
/// `args` contains additional journalctl arguments such as filters.
pub fn read<I, S>(journal: Journal, args: I) -> Vec<HashMap<String, FieldValue>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new("journalctl");
    if matches!(journal, Journal::User) {
        command.arg("--user");
    }

    let stdout = String::from_utf8(
        command
            .arg("--output=json")
            // We pass --all to circumvent journalctl's default limit of 4096 bytes for field values
            .arg("--all")
            .args(args)
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap();

    stdout
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

// Read from the journal of the current process.
pub fn read_current_process(target: &str) -> Vec<HashMap<String, FieldValue>> {
    // Filter by the PID of the current test process and the module path
    read(
        Journal::User,
        vec![
            format!("_PID={}", std::process::id()),
            format!("TARGET={}", target),
        ],
    )
}

pub fn read_one_entry(target: &str) -> HashMap<String, FieldValue> {
    retry::retry(Fixed::from_millis(100).take(30), || {
        let mut entries = read_current_process(target);
        if entries.len() == 1 {
            Ok(entries.pop().unwrap())
        } else {
            Err(format!("No entries in journal for target {}", target))
        }
    })
    .unwrap()
}
