mod eventmsgs;

use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

use windows_registry::LOCAL_MACHINE;

use windows::{
  Win32::{
    Foundation::HANDLE,
    System::EventLog::{
      DeregisterEventSource, EVENTLOG_ERROR_TYPE, EVENTLOG_INFORMATION_TYPE,
      EVENTLOG_WARNING_TYPE, RegisterEventSourceW, ReportEventW
    }
  },
  core::PCWSTR
};

use crate::eventmsgs::{
  MSG_DEBUG, MSG_ERROR, MSG_INFO, MSG_TRACE, MSG_WARNING
};

const REG_BASEKEY: &str =
  r"SYSTEM\CurrentControlSet\Services\EventLog\Application";

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("Could not determine executable path")]
  ExePathNotFound,

  #[error("Call to RegisterEventSource failed")]
  RegisterSourceFailed(#[from] windows::core::Error),

  #[error("Failed to modify registry key or value")]
  Registry(#[from] windows_result::Error)
}

pub struct EventLog {
  handle: HANDLE,
  level: log::Level
}

unsafe impl Send for EventLog {}
unsafe impl Sync for EventLog {}

#[inline]
fn win_string(s: &str) -> Vec<u16> {
  OsStr::new(s).encode_wide().chain(once(0)).collect()
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
  #[error("Failed to create logger")]
  Create(#[from] Error),

  #[error("Failed to set logger")]
  Set(#[from] SetLoggerError)
}

/// # Errors
pub fn init(name: &str, level: log::Level) -> Result<(), InitError> {
  let logger = Box::new(EventLog::new(name, level)?);
  log::set_boxed_logger(logger)
    .map(|()| log::set_max_level(LevelFilter::Trace))?;
  Ok(())
}

/// # Errors
pub fn deregister(name: &str) -> Result<(), windows_result::Error> {
  let key = LOCAL_MACHINE.open(REG_BASEKEY)?;
  key.remove_tree(name)
}

/// # Errors
pub fn register(name: &str) -> Result<(), Error> {
  let current_exe =
    std::env::current_exe().map_err(|_| Error::ExePathNotFound)?;
  let exe_path = current_exe.to_str().ok_or(Error::ExePathNotFound)?;

  let base_key = LOCAL_MACHINE.open(REG_BASEKEY).map_err(Error::Registry)?;

  let app_key = base_key.create(name).map_err(Error::Registry)?;

  app_key
    .set_string("EventMessageFile", exe_path)
    .map_err(Error::Registry)?;

  Ok(())
}

impl EventLog {
  /// # Errors
  pub fn new(name: &str, level: log::Level) -> Result<Self, Error> {
    let wide_name = win_string(name);
    let handle =
      unsafe { RegisterEventSourceW(None, PCWSTR(wide_name.as_ptr()))? };

    Ok(Self { handle, level })
  }
}

impl Drop for EventLog {
  fn drop(&mut self) {
    let _ = unsafe { DeregisterEventSource(self.handle) };
  }
}

impl log::Log for EventLog {
  #[inline]
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
      Level::Trace => (EVENTLOG_INFORMATION_TYPE, MSG_TRACE)
    };

    let msg = win_string(&format!("{}", record.args()));
    let vec = vec![PCWSTR(msg.as_ptr())];

    unsafe {
      let _ = ReportEventW(self.handle, ty, 0, id, None, 0, Some(&vec), None);
    };
  }

  fn flush(&self) {}
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
