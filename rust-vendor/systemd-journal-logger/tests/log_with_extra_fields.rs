// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings, clippy::all)]

mod journal;

use similar_asserts::assert_eq;
use systemd_journal_logger::JournalLog;

#[derive(Debug)]
struct SomeDummy {
    #[allow(dead_code)]
    foo: usize,
}

#[test]
fn log_with_extra_fields() {
    JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(log::LevelFilter::Info);

    let error = std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "not found: the important file",
    );
    let dummy = SomeDummy { foo: 42 };

    log::error!(target: "log_with_extra_fields", dummy:?, spam = "no eggs", error:err; "Hello World");

    let entry = journal::read_one_entry("log_with_extra_fields");
    assert_eq!(entry["MESSAGE"], "Hello World");
    assert_eq!(entry["SPAM"], "no eggs");
    assert_eq!(entry["ERROR"], "not found: the important file");
    assert_eq!(entry["DUMMY"], format!("{:?}", dummy));
}
