# systemd-journal-logger

A pure Rust [log] logger for the [systemd journal][1].

[log]: https://docs.rs/log
[1]: https://www.freedesktop.org/software/systemd/man/systemd-journald.service.html

## Usage

```console
$ cargo add systemd-journal-logger
```

Then initialize the logger at the start of `main`:

```rust
use log::{info, warn, error, LevelFilter};
use systemd_journal_logger::JournalLog;

JournalLog::new().unwrap().install().unwrap();
log::set_max_level(LevelFilter::Info);

info!("hello log");
warn!("warning");
error!("oops");
```

You can also add additional fields to every log message, such as the version of your executable:

```rust
use log::{info, warn, error, LevelFilter};
use systemd_journal_logger::JournalLog;

JournalLog::new()
    .unwrap()
    .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
    .with_syslog_identifier("foo".to_string())
    .install().unwrap();
log::set_max_level(LevelFilter::Info);

info!("this message has an extra VERSION field in the journal");
```

These extra fields appear in the output of `journalctl --output=verbose` or in any of the JSON output formats of `journalctl`.

See [systemd_service.rs](./examples/systemd_service.rs) for a simple example of logging in a systemd service which automatically falls back to a different logger if not started through systemd.

## Related projects

- [rust-systemd](https://github.com/jmesmon/rust-systemd) provides a [logger implementation][1] based on the `libsystemd` C library.
- [slog-journald](https://github.com/slog-rs/journald) provides an [slog] logger for the systemd journal, also based on the `libsystemd` C library.
- [tracing-journald](https://github.com/tokio-rs/tracing/tree/master/tracing-journald) provides a tracing backend for the systemd journal, in a pure Rust implementation.

Both loggers use mostly the same fields and priorities as this implementation.

[1]: https://docs.rs/systemd/0.8.2/systemd/journal/struct.JournalLog.html
[slog]: https://github.com/slog-rs/slog

## Minimum Supported Rust Version

The MSRV used in this repo is best effort only and may be bumped at any time, even in patch releases.

## License

Either [MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE-2.0), at your option.
