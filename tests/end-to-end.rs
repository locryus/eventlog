#[macro_use]
extern crate log;

use std::{process::Command, str};

use log::Level;

use eventlog::{deregister, init, register};

use rand::{RngExt, distr::Alphanumeric};


#[test]
fn end_to_end() {
  let mut rng = rand::rng();
  let rand_string: String =
    (0..16).map(|_| rng.sample(Alphanumeric) as char).collect();
  let log_source = format!("eventlog-test-{rand_string}");

  // Add log source to Windows registry
  register(&log_source).unwrap();

  // Do some logging and verification
  init(&log_source, Level::Trace).unwrap();
  log_and_verify_one(Level::Error, &log_source, "Error", "1", "Error!!");
  log_and_verify_one(Level::Warn, &log_source, "Warning", "2", "Warning!!");
  log_and_verify_one(Level::Info, &log_source, "Information", "3", "Info!!");
  log_and_verify_one(Level::Debug, &log_source, "Information", "4", "Debug!!");
  log_and_verify_one(Level::Trace, &log_source, "Information", "5", "Trace!!");

  // Remove log source from Windows registry
  deregister(&log_source).unwrap();
}

fn log_and_verify_one(
  level: Level,
  log_source: &str,
  entry_type: &str,
  entry_id: &str,
  msg: &str
) {
  log!(level, "{msg}");

  // Use PowerShell to extract formatted entries from the event log.
  let mut command = Command::new("powershell");
  command.arg("-Command").arg(format!(
    "Get-EventLog -Newest 1 -LogName Application -Source {log_source} | \
     Select-Object Source, EntryType, EventID, Message | foreach {{ \"$_\" }}"
  ));
  let out = command.output().unwrap();

  assert_eq!(
    format!(
      "@{{Source={}; EntryType={}; EventID={}; Message={}}}\r\n",
      &log_source, &entry_type, entry_id, msg
    ),
    str::from_utf8(&out.stdout).unwrap()
  );
}

// vim: set ft=rust et sw=2 ts=2 sts=2 cinoptions=2 tw=79 :
