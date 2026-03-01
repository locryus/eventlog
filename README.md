# eventlog

[![Latest version](https://img.shields.io/crates/v/eventlog.svg)](https://crates.io/crates/eventlog)
[![Documentation](https://docs.rs/eventlog/badge.svg)](https://docs.rs/eventlog)

A [log](https://docs.rs/log)-compatible wrapper around the [Windows Event Log API](https://docs.microsoft.com/en-us/windows/desktop/eventlog/event-logging).

## Genealogy

- Jeroen C. van Gelderen created the original [winlog](https://gitlab.com/arbitrix/winlog) project.
- Brendan Molloy created the `eventlog` fork from `winlog`.

## Features

* Writes Rust log messages to the Windows event log using the
  [RegisterEventSourceW](https://docs.microsoft.com/en-us/windows/desktop/api/Winbase/nf-winbase-registereventsourcew)
  and [ReportEventW](https://docs.microsoft.com/en-us/windows/desktop/api/winbase/nf-winbase-reporteventw) APIs.
* Provides utility functions to register/unregister your
  [event source](https://docs.microsoft.com/en-us/windows/desktop/eventlog/event-sources) in the Windows registry.
* Embeds a small (120-byte) message resource library containing the necessary log message templates in your executable.
* Does not panic.

The five Rust log levels are mapped to Windows [event types](https://docs.microsoft.com/en-us/windows/desktop/eventlog/event-types) as follows:

| Rust Log Level | Windows Event Type | Windows Event Id |
| -------------- | ------------------ | ---------------- |
| Error          | Error              | 1                |
| Warn           | Warning            | 2                |
| Info           | Informational      | 3                |
| Debug          | Informational      | 4                |
| Trace          | Informational      | 5                |


## Requirements

* Rust 1.29+ on Windows with MSVC toolchain
* [Windows, optional] [mc.exe](https://docs.microsoft.com/en-us/windows/desktop/wes/message-compiler--mc-exe-) and [rc.exe](https://docs.microsoft.com/en-us/windows/desktop/menurc/resource-compiler) (only required when `eventmsgs.mc` is changed)
* [Windows, optional] PowerShell (used for the end-to-end test)

## Usage

### Register log source with Windows

Register the log source in the Windows registry:
```rust
eventlog::register("Example Log").unwrap();
```

This usually requires **Administrator** permission so this is usually done during installation time.

If your MSI installer (or similar) registers your event sources you should not call this.

### Log events

```rust
eventlog::init("Example Log", log::Level::Trace).unwrap();

info!("Hello, Event Log");
trace!("This will be logged too");
```

### Deregister log source

Deregister the log source: 

```rust
eventlog::deregister("Example Log").unwrap();
```

This is usually done during program uninstallation. If your MSI installer (or similar) deregisters your event sources you should not call this.

### Internals

Artifacts `eventmsgs.lib` and `eventmsgs.rs` are under source control so users 
don't need to have `mc.exe` and `rc.exe` installed for a standard build.

1. If `build.rs` determines that `eventmsgs.mc` was changed then `build.rs`:
   * invokes `mc.exe` (which creates `eventmsgs.h`)
   * invokes `rc.exe` (which creates `eventmsgs.lib`)
   * creates `eventmsgs.rs` from `eventmsgs.h`.
2. `build.rs` emits linker flags so `eventmsgs.lib` can found.
3. Standard `cargo build` follows.


## Testing

The end-to-end test requires 'Full Control' permissions on the 
`HKLM\SYSTEM\CurrentControlSet\Services\EventLog\Application`
registry key.

```rust
cargo test
```

### Process:
1. Create a unique temporary event source name (`eventlog-test-###########`).
2. Register our compiled test executable as ```EventMessageFile``` for 
   the event source in the Windows registry. You can see a new key at 
   `HKLM\SYSTEM\CurrentControlSet\Services\EventLog\Application\eventlog-test-###########`.
2. Write some log messages to the event source.
3. Use PowerShell to retrieve the logged messages.
4. Deregister our event source. This removes the `eventlog-test-###########` 
   registry key.
5. Assert that the retrieved log messages are correct. 


### See also

- [`fltevtlog`](https://crates.io/crates/fltevtlog) is a wrapper around
  `eventlog` that supports `env_logger`'s per-module log level configuration.

## License

Licensed under either of

* Apache License, Version 2.0 (LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license (LICENSE-MIT or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in the work by you, as defined in the Apache-2.0 license, shall 
be dual licensed as above, without any additional terms or conditions.
