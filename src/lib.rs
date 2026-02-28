#![cfg_attr(docsrs, feature(doc_cfg))]

mod eventmsgs;

#[cfg(feature = "filtering")]
#[cfg_attr(docsrs, doc(cfg(feature = "filtering")))]
pub mod filtering;

use std::convert::TryInto;
use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};
use registry::{Data, Hive, Security};
use windows::core::PCWSTR;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::EventLog::{
    DeregisterEventSource, RegisterEventSourceW, ReportEventW, EVENTLOG_ERROR_TYPE,
    EVENTLOG_INFORMATION_TYPE, EVENTLOG_WARNING_TYPE,
};

use crate::eventmsgs::{MSG_DEBUG, MSG_ERROR, MSG_INFO, MSG_TRACE, MSG_WARNING};

const REG_BASEKEY: &str = r"SYSTEM\CurrentControlSet\Services\EventLog\Application";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Could not determine executable path")]
    ExePathNotFound,

    #[error("Call to RegisterEventSource failed")]
    RegisterSourceFailed(#[from] windows::core::Error),

    #[error("Failed to modify registry key")]
    RegKey(#[from] registry::key::Error),

    #[error("Failed to modify registry value")]
    RegValue(#[from] registry::value::Error),
}

pub struct EventLog {
    handle: HANDLE,
    level: log::Level,
}

unsafe impl Send for EventLog {}
unsafe impl Sync for EventLog {}

#[inline(always)]
fn win_string(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(once(0)).collect()
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Failed to create logger")]
    Create(#[from] Error),

    #[error("Failed to set logger")]
    Set(#[from] SetLoggerError),
}

pub fn init(name: &str, level: log::Level) -> Result<(), InitError> {
    let logger = Box::new(EventLog::new(name, level)?);
    log::set_boxed_logger(logger).map(|()| log::set_max_level(LevelFilter::Trace))?;
    Ok(())
}

pub fn deregister(name: &str) -> Result<(), registry::key::Error> {
    let key = Hive::LocalMachine.open(REG_BASEKEY, Security::Read)?;
    key.delete(name, true)
}

pub fn register(name: &str) -> Result<(), Error> {
    let current_exe = std::env::current_exe().map_err(|_| Error::ExePathNotFound)?;
    let exe_path = current_exe.to_str().ok_or(Error::ExePathNotFound)?;

    let key = Hive::LocalMachine.open(REG_BASEKEY, Security::Write)?;
    let app_key = key.create(name, Security::Write)?;
    Ok(app_key.set_value(
        "EventMessageFile",
        &Data::String(exe_path.try_into().map_err(|_| Error::ExePathNotFound)?),
    )?)
}

impl EventLog {
    pub fn new(name: &str, level: log::Level) -> Result<EventLog, Error> {
        let wide_name = win_string(name);
        let handle = unsafe { RegisterEventSourceW(None, PCWSTR(wide_name.as_ptr()))? };

        Ok(EventLog { handle, level })
    }
}

impl Drop for EventLog {
    fn drop(&mut self) {
        let _ = unsafe { DeregisterEventSource(self.handle) };
    }
}

impl log::Log for EventLog {
    #[inline(always)]
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let (ty, id) = match record.level() {
            Level::Error => (EVENTLOG_ERROR_TYPE, MSG_ERROR),
            Level::Warn => (EVENTLOG_WARNING_TYPE, MSG_WARNING),
            Level::Info => (EVENTLOG_INFORMATION_TYPE, MSG_INFO),
            Level::Debug => (EVENTLOG_INFORMATION_TYPE, MSG_DEBUG),
            Level::Trace => (EVENTLOG_INFORMATION_TYPE, MSG_TRACE),
        };

        let msg = win_string(&format!("{}", record.args()));
        let vec = vec![PCWSTR(msg.as_ptr())];

        unsafe {
            let _ = ReportEventW(self.handle, ty, 0, id, None, 0, Some(&vec), None);
        };
    }

    fn flush(&self) {}
}
