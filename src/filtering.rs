use env_logger::Logger;
use log::{Log, Metadata, Record};

pub use env_logger::Builder;

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
/// Use `builder` to configure filtering.
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
