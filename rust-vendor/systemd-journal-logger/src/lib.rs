// Copyright Sebastian Wiesner <sebastian@swsnr.de>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A pure Rust [log] logger for the [systemd journal][1].
//!
//! [log]: https://docs.rs/log
//! [1]: https://www.freedesktop.org/software/systemd/man/journalctl.html
//!
//! # Usage
//!
//! Create a [`JournalLog`] with [`JournalLog::new`] and then use [`JournalLog::install`] to
//! setup journal logging.  Then configure the logging level and now you can use the standard macros
//! from the [`log`] crate to send log messages to the systemd journal:
//!
//! ```rust
//! use log::{info, warn, error, LevelFilter};
//! use systemd_journal_logger::JournalLog;
//!
//! JournalLog::new().unwrap().install().unwrap();
//! log::set_max_level(LevelFilter::Info);
//!
//! info!("hello log");
//! warn!("warning");
//! error!("oops");
//! ```
//!
//! See [`JournalLog`] for details about the logging format.
//!
//! ## Journal connections
//!
//! In a service you can use [`connected_to_journal`] to check whether
//! the standard output or error stream of the current process is directly
//! connected to the systemd journal (the default for services started by
//! systemd) and fall back to logging to standard error if that's not the
//! case.  Take a look at the [systemd_service.rs] example for details.
//!
//! [systemd_service.rs]: https://codeberg.org/swsnr/systemd-journal-logger.rs/src/branch/main/examples/systemd_service.rs
//!
//! ```rust
//! use log::{info, warn, error, LevelFilter};
//! use systemd_journal_logger::JournalLog;
//!
//! JournalLog::new()
//!     .unwrap()
//!     .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
//!     .with_syslog_identifier("foo".to_string())
//!     .install().unwrap();
//! log::set_max_level(LevelFilter::Info);
//!
//! info!("this message has an extra VERSION field in the journal");
//! ```
//!
//! You can display these extra fields with `journalctl --output=verbose` and extract them with any of the structured
//! output formats of `journalctl`, e.g. `journalctl --output=json`.

#![deny(warnings, missing_docs, clippy::all)]
#![forbid(unsafe_code)]

use std::io::prelude::*;
use std::os::fd::AsFd;

use client::JournalClient;
use log::kv::{Error, Key, Value, VisitSource};
use log::{Level, Log, Metadata, Record, SetLoggerError};

mod client;
mod fields;

use fields::*;

/// Whether the current process is directly connected to the systemd journal.
///
/// Return `true` if the device and inode numbers of the [`std::io::stderr`]
/// file descriptor match the value of `$JOURNAL_STREAM` (see `systemd.exec(5)`).
/// Otherwise, return `false`.
pub fn connected_to_journal() -> bool {
    rustix::fs::fstat(std::io::stderr().as_fd())
        .map(|stat| format!("{}:{}", stat.st_dev, stat.st_ino))
        .ok()
        .and_then(|stderr| {
            std::env::var_os("JOURNAL_STREAM").map(|s| s.to_string_lossy() == stderr.as_str())
        })
        .unwrap_or(false)
}

/// Create a syslog identifier from the current executable.
///
/// Return `None` if we're unable to determine the name, e.g. because
/// [`std::env::current_exe`] failed or returned some weird name.
pub fn current_exe_identifier() -> Option<String> {
    let executable = std::env::current_exe().ok()?;
    Some(executable.file_name()?.to_string_lossy().into_owned())
}

struct WriteKeyValues<'a>(&'a mut Vec<u8>);

impl<'kvs> VisitSource<'kvs> for WriteKeyValues<'_> {
    fn visit_pair(&mut self, key: Key<'kvs>, value: Value<'kvs>) -> Result<(), Error> {
        put_field_length_encoded(self.0, FieldName::WriteEscaped(key.as_str()), value);
        Ok(())
    }
}

/// A systemd journal logger.
///
/// ## Journal access
///
/// ## Standard fields
///
/// The journald logger always sets the following standard [journal fields]:
///
/// - `PRIORITY`: The log level mapped to a priority (see below).
/// - `MESSAGE`: The formatted log message (see [`log::Record::args()`]).
/// - `SYSLOG_PID`: The PID of the running process (see [`std::process::id()`]).
/// - `CODE_FILE`: The filename the log message originates from (see [`log::Record::file()`], only if present).
/// - `CODE_LINE`: The line number the log message originates from (see [`log::Record::line()`], only if present).
///
/// It also sets `SYSLOG_IDENTIFIER` if non-empty (see [`JournalLog::with_syslog_identifier`]).
///
/// Additionally it also adds the following non-standard fields:
///
/// - `TARGET`: The target of the log record (see [`log::Record::target()`]).
/// - `CODE_MODULE`: The module path of the log record (see [`log::Record::module_path()`], only if present).
///
/// [journal fields]: https://www.freedesktop.org/software/systemd/man/systemd.journal-fields.html
///
/// ## Log levels and Priorities
///
/// [`log::Level`] gets mapped to journal (syslog) priorities as follows:
///
/// - [`Level::Error`] → `3` (err)
/// - [`Level::Warn`] → `4` (warning)
/// - [`Level::Info`] → `5` (notice)
/// - [`Level::Debug`] → `6` (info)
/// - [`Level::Trace`] → `7` (debug)
///
/// Higher priorities (crit, alert, and emerg) are not used.
///
/// ## Custom fields and structured record fields
///
/// In addition to these fields the logger also adds all structures key-values
/// (see [`log::Record::key_values`]) from each log record as journal fields,
/// and also supports global extra fields via [`Self::with_extra_fields`].
///
/// Journald allows only ASCII uppercase letters, ASCII digits, and the
/// underscore in field names, and limits field names to 64 bytes.  See upstream's
/// [`journal_field_valid`][jfv] for the precise validation rules.
///
/// This logger mangles the keys of additional key-values on records and names
/// of custom fields according to the following rules, to turn them into valid
/// journal fields:
///
/// - If the key is entirely empty, use `EMPTY`.
/// - Transform the entire value to ASCII uppercase.
/// - Replace all invalid characters with underscore.
/// - If the key starts with an underscore or digit, which is not permitted,
///   prepend `ESCAPED_`.
/// - Cap the result to 64 bytes.
///
/// [jfv]: https://github.com/systemd/systemd/blob/a8b53f4f1558b17169809effd865232580e4c4af/src/libsystemd/sd-journal/journal-file.c#L1698
///
/// # Errors
///
/// The logger tries to connect to journald when constructed, to provide early
/// on feedback if journald is not available (e.g. in containers where the
/// journald socket is not mounted into the container).
///
/// Later on, the logger simply ignores any errors when sending log records to
/// journald, simply because the log interface does not expose faillible operations.
pub struct JournalLog {
    /// The journald client
    client: JournalClient,
    /// Preformatted extra fields to be appended to every log message.
    extra_fields: Vec<u8>,
    /// The syslog identifier.
    syslog_identifier: String,
}

fn record_payload(syslog_identifier: &str, record: &Record) -> Vec<u8> {
    use FieldName::*;
    let mut buffer = Vec::with_capacity(1024);
    // Write standard fields. Numeric fields can't contain new lines so we
    // write them directly, everything else goes through the put functions
    // for property mangling and length-encoding
    let priority = match record.level() {
        Level::Error => b"3",
        Level::Warn => b"4",
        Level::Info => b"5",
        Level::Debug => b"6",
        Level::Trace => b"7",
    };
    put_field_bytes(&mut buffer, WellFormed("PRIORITY"), priority);
    put_field_length_encoded(&mut buffer, WellFormed("MESSAGE"), record.args());
    // Syslog compatibility fields
    writeln!(&mut buffer, "SYSLOG_PID={}", std::process::id()).unwrap();
    if !syslog_identifier.is_empty() {
        put_field_bytes(
            &mut buffer,
            WellFormed("SYSLOG_IDENTIFIER"),
            syslog_identifier.as_bytes(),
        );
    }
    if let Some(file) = record.file() {
        put_field_bytes(&mut buffer, WellFormed("CODE_FILE"), file.as_bytes());
    }
    if let Some(module) = record.module_path() {
        put_field_bytes(&mut buffer, WellFormed("CODE_MODULE"), module.as_bytes());
    }
    if let Some(line) = record.line() {
        writeln!(&mut buffer, "CODE_LINE={}", line).unwrap();
    }
    put_field_bytes(
        &mut buffer,
        WellFormed("TARGET"),
        record.target().as_bytes(),
    );
    // Put all structured values of the record
    record
        .key_values()
        .visit(&mut WriteKeyValues(&mut buffer))
        .unwrap();
    buffer
}

impl JournalLog {
    /// Create a journal log instance with a default syslog identifier.
    pub fn new() -> std::io::Result<Self> {
        let logger = Self::empty()?;
        Ok(logger.with_syslog_identifier(current_exe_identifier().unwrap_or_default()))
    }

    /// Create an empty journal log instance, with no extra fields and no syslog
    /// identifier.
    ///
    /// See [`Self::with_syslog_identifier`] and [`Self::with_extra_fields`] to
    /// set either.  It's recommended to at least set the syslog identifier.
    pub fn empty() -> std::io::Result<Self> {
        Ok(Self {
            client: JournalClient::new()?,
            extra_fields: Vec::new(),
            syslog_identifier: String::new(),
        })
    }

    /// Install this logger globally.
    ///
    /// See [`log::set_boxed_logger`].
    pub fn install(self) -> Result<(), SetLoggerError> {
        log::set_boxed_logger(Box::new(self))
    }

    /// Add an extra field to be added to every log entry.
    ///
    /// `name` is the name of a custom field, and `value` its value.  Fields are
    /// appended to every log entry, in order they were added to the logger.
    ///
    /// ## Restrictions on field names
    ///
    /// `name` should be a valid journal file name, i.e. it must only contain
    /// ASCII uppercase alphanumeric characters and the underscore, and must
    /// start with an ASCII uppercase letter.
    ///
    /// Invalid keys in `extra_fields` are escaped according to the rules
    /// documented in [`JournalLog`].
    ///
    /// It is not recommended that `name` is any of the standard fields already
    /// added by this logger (see [`JournalLog`]); though journald supports
    /// multiple values for a field, journald clients may not handle unexpected
    /// multi-value fields properly and perhaps only show the first value.
    /// Specifically, even `journalctl` will only shouw the first `MESSAGE` value
    /// of journal entries.
    ///
    /// ## Restrictions on values
    ///
    /// There are no restrictions on the value.
    pub fn add_extra_field<K: AsRef<str>, V: AsRef<[u8]>>(mut self, name: K, value: V) -> Self {
        put_field_bytes(
            &mut self.extra_fields,
            FieldName::WriteEscaped(name.as_ref()),
            value.as_ref(),
        );
        self
    }

    /// Set extra fields to be added to every log entry.
    ///
    /// Remove all previously added fields.
    ///
    /// See [`Self::add_extra_field`] for details.
    pub fn with_extra_fields<I, K, V>(mut self, extra_fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<[u8]>,
    {
        self.extra_fields.clear();
        let mut logger = self;
        for (name, value) in extra_fields {
            logger = logger.add_extra_field(name, value);
        }
        logger
    }

    /// Set the given syslog identifier for this logger.
    ///
    /// The logger writes this string in the `SYSLOG_IDENTIFIER` field, which
    /// can be filtered for with `journalctl -t`.
    ///
    /// Use [`current_exe_identifier()`] to obtain the standard identifier for
    /// the current executable.
    pub fn with_syslog_identifier(mut self, identifier: String) -> Self {
        self.syslog_identifier = identifier;
        self
    }

    /// Get the complete journal payload for `record`, including extra fields
    /// from this logger.
    fn record_payload(&self, record: &Record) -> Vec<u8> {
        let mut payload = record_payload(&self.syslog_identifier, record);
        payload.extend_from_slice(&self.extra_fields);
        payload
    }

    /// Send a single log record to the journal.
    ///
    /// Extract all fields (standard and custom) from `record` (`see [`JournalLog`]),
    /// append all `extra_fields` given to this logger, and send the result to
    /// journald.
    pub fn journal_send(&self, record: &Record) -> std::io::Result<()> {
        let _ = self.client.send_payload(&self.record_payload(record))?;
        Ok(())
    }
}

/// The [`Log`] interface for [`JournalLog`].
impl Log for JournalLog {
    /// Whether this logger is enabled.
    ///
    /// Always returns `true`.
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    /// Send the given `record` to the systemd journal.
    ///
    /// # Errors
    ///
    /// Ignore any errors which occur when sending `record` to journald because
    /// we cannot reasonably handle them at this place.
    ///
    /// See [`JournalLog::journal_send`] for a function which returns any error
    /// which might have occurred while sending the `record` to the journal.
    fn log(&self, record: &Record) {
        // We can't really handle errors here, so simply discard them.
        // The alternative would be to panic, but a failed logging call should
        // not bring the entire process down.
        let _ = self.journal_send(record);
    }

    /// Flush log records.
    ///
    /// A no-op for journal logging.
    fn flush(&self) {}
}
