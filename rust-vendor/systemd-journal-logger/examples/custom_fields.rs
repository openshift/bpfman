// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![deny(warnings, clippy::all)]

//! Add custom fields to every emitted log record.
//!
//! This example demonstrates how to add arbitrary extra fields to log records
//! emitted to the journald.
//!
//! This approach extends the key/value pairs attached to a log record with new
//! pairs, in a generic way, which works with any underlying logger
//! implementation.

use log::kv::ToValue;
use log::Log;
use systemd_journal_logger::JournalLog;

/// A logger which adds the thread ID to every log record.
struct AddThreadId<L: Log>(L);

impl<L: Log> Log for AddThreadId<L> {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.0.enabled(metadata)
    }

    fn log(&self, record: &log::Record) {
        let thread_id = format!("{:?}", std::thread::current().id());
        let extra_fields = &[("THREAD_ID", thread_id.to_value())];
        self.0.log(
            &record
                .to_builder()
                .key_values(&[record.key_values(), &extra_fields])
                .build(),
        );
    }

    fn flush(&self) {
        self.0.flush();
    }
}

fn main() {
    let log = AddThreadId(
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))]),
    );
    log::set_boxed_logger(Box::new(log)).unwrap();
    log::set_max_level(log::LevelFilter::Trace);

    log::info!(hello = "world"; "A info message");
}
