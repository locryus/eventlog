# Changelog

⚠️  indicates a breaking change.

## [Unreleased]

### Added

### Changed

### Removed

---

## [0.4.0] - 2026-03-05

### Changed

- Project has a new home.  Many thanks to Brendan Molloy for his work on
  `eventlog` and for handing passing on the torch when he did not want to
  maintain it any longer.
- ⚠️ Switch from discontinued `registry` crate to `windows-registry`.
- Set MSRV to `1.58`.
- If `ReportEvent()` fails, write error message to debugger console through
  `OutputDebugString()`.
- Configure clippy to be more pedantic.
- Deprecation fix: rename `.cargo/config` to `.cargo/config.toml`.

