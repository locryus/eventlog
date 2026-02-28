//! Support for `env_logger`'s filtering engine.
//!
//! This submodule provides a simple wrapper around [`EventLog`](super::EventLog) and
//! `env_logger`'s filtering engine.

use env_logger::Logger;
use log::{Log, Metadata, Record};

// Re-export env_logger, but also provide a shortcut to its `Builder`, because
// realistically speaking the use-case will probably in the majority of cases
// be something along the line of:
//
// init(Builder::from_env("MY_LOG"), "myservice", log::Level::Trace);
pub use env_logger::{self, Builder};

use crate::{Error, InitError};

/// A variant of [`EventLog`](super::EventLog) that uses `env_logger`'s filtering engine.
pub struct FilteredEventLog {
    filter_logger: Logger,
    inner: crate::EventLog,
}

impl Log for FilteredEventLog {
    fn enabled(&self, metadata: &Metadata) -> bool {
        // Check the env_logger-style filter rules
        self.filter_logger.enabled(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            self.inner.log(record);
        }
    }

    fn flush(&self) {
        self.inner.flush();
    }
}

/// Initialize the filtering event logger.
///
/// The `builder` can be used to configure module-level filtering.
///
/// ```no_run
/// use eventlog::filtering::{Builder, init};
///
/// let bldr = Builder::from_env("MY_LOG");
/// init(bldr, "myservice", log::LevelFilter::Trace).unwrap();
/// ```
pub fn init(mut builder: Builder, name: &str, level: log::Level) -> Result<(), InitError> {
    let filter_logger = builder.build();

    let logger = FilteredEventLog {
        filter_logger,
        inner: crate::EventLog::new(name, level)?,
    };

    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(log::LevelFilter::Trace))?;

    Ok(())
}
