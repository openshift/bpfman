// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Write well-formated journal fields to buffers.

use std::fmt::Arguments;
use std::io::Write;

use log::kv::Value;

pub enum FieldName<'a> {
    WellFormed(&'a str),
    WriteEscaped(&'a str),
}

/// Whether `c` is a valid character in the key of a journal field.
///
/// Journal field keys may only contain ASCII uppercase letters A to Z,
/// numbers 0 to 9 and the underscore.
fn is_valid_key_char(c: char) -> bool {
    matches!(c, 'A'..='Z' | '0'..='9' | '_')
}

/// Escape a `key` for use in a systemd journal field.
///
/// See [`crate::JournalLog`] for these rules.
pub fn escape_journal_key(key: &str) -> Vec<u8> {
    let mut escaped = key
        .to_ascii_uppercase()
        .replace(|c| !is_valid_key_char(c), "_");
    if escaped.starts_with(|c: char| matches!(c, '_' | '0'..='9')) {
        escaped = format!("ESCAPED_{}", escaped);
    }
    let mut payload = escaped.into_bytes();
    payload.truncate(64);
    payload
}

fn put_field_name(buffer: &mut Vec<u8>, name: FieldName<'_>) {
    match name {
        FieldName::WellFormed(name) => buffer.extend_from_slice(name.as_bytes()),
        FieldName::WriteEscaped("") => buffer.extend_from_slice(b"EMPTY"),
        // FIXME: We should try to find a way to do this with less allocations.
        FieldName::WriteEscaped(name) => buffer.extend_from_slice(&escape_journal_key(name)),
    }
}

pub trait PutAsFieldValue {
    fn put_field_value(self, buffer: &mut Vec<u8>);
}

impl PutAsFieldValue for &[u8] {
    fn put_field_value(self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(self)
    }
}

impl PutAsFieldValue for &Arguments<'_> {
    fn put_field_value(self, buffer: &mut Vec<u8>) {
        match self.as_str() {
            Some(s) => buffer.extend_from_slice(s.as_bytes()),
            None => write!(buffer, "{}", self).unwrap(),
        }
    }
}

impl PutAsFieldValue for Value<'_> {
    fn put_field_value(self, buffer: &mut Vec<u8>) {
        // TODO: We can probably write the value more efficiently by visiting it?
        write!(buffer, "{}", self).unwrap()
    }
}

pub fn put_field_length_encoded<V: PutAsFieldValue>(
    buffer: &mut Vec<u8>,
    name: FieldName<'_>,
    value: V,
) {
    put_field_name(buffer, name);
    buffer.push(b'\n');
    // Reserve the length tag
    buffer.extend_from_slice(&[0; 8]);
    let value_start = buffer.len();
    value.put_field_value(buffer);
    let value_end = buffer.len();
    // Fill the length tag
    let length_bytes = ((value_end - value_start) as u64).to_le_bytes();
    buffer[value_start - 8..value_start].copy_from_slice(&length_bytes);
    buffer.push(b'\n');
}

pub fn put_field_bytes(buffer: &mut Vec<u8>, name: FieldName<'_>, value: &[u8]) {
    if value.contains(&b'\n') {
        // Write as length encoded field
        put_field_length_encoded(buffer, name, value);
    } else {
        put_field_name(buffer, name);
        buffer.push(b'=');
        buffer.extend_from_slice(value);
        buffer.push(b'\n');
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use similar_asserts::assert_eq;
    use FieldName::*;

    #[test]
    fn escape_journal_key() {
        for case in &["FOO", "FOO_123"] {
            assert_eq!(
                &String::from_utf8_lossy(&super::escape_journal_key(case)),
                case
            );
        }

        let cases = vec![
            ("foo", "FOO"),
            ("_foo", "ESCAPED__FOO"),
            ("1foo", "ESCAPED_1FOO"),
            ("Hallöchen", "HALL_CHEN"),
        ];
        for (key, expected) in cases {
            assert_eq!(
                &String::from_utf8_lossy(&super::escape_journal_key(key)),
                expected
            );
        }
    }

    #[test]
    fn put_field_length_encoded() {
        let mut buffer = Vec::new();
        // See "Data Format" in https://systemd.io/JOURNAL_NATIVE_PROTOCOL/ for this example
        super::put_field_length_encoded(&mut buffer, WellFormed("FOO"), "BAR".as_bytes());
        assert_eq!(&buffer, b"FOO\n\x03\0\0\0\0\0\0\0BAR\n");
    }

    #[test]
    fn put_field_bytes_no_newline() {
        let mut buffer = Vec::new();
        super::put_field_bytes(&mut buffer, WellFormed("FOO"), "BAR".as_bytes());
        assert_eq!(&buffer, b"FOO=BAR\n");
    }

    #[test]
    fn put_field_bytes_newline() {
        let mut buffer = Vec::new();
        super::put_field_bytes(
            &mut buffer,
            WellFormed("FOO"),
            "BAR\nSPAM_WITH_EGGS".as_bytes(),
        );
        assert_eq!(&buffer, b"FOO\n\x12\0\0\0\0\0\0\0BAR\nSPAM_WITH_EGGS\n");
    }
}
