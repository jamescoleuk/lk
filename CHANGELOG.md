# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and this project adheres to [Semantic Versioning](https://semver.org/).

## [2026-08-06] - 0.7.0

### Removed

- The `--fuzzy` mode and its `--number` option. It confused things: the TUI
  has the same fuzzy search plus details, and it's the default. If your
  `lk.toml` still says `default_mode = "fuzzy"` you'll get the TUI and a
  polite nudge to update it

### Changed

- Modernised the codebase: Rust edition 2024 and updated all dependencies
- Replaced the deprecated `structopt` with `clap` 4 (adds `--version`/`-V`)
- Upgraded `ratatui` to 0.30 and dropped the direct `crossterm`/`termion`/`fuzzy_finder` dependencies
- Replaced `log4rs` with `simplelog`, and removed the unused `toml`, `serde`, `pad` and `nanoid` dependencies
- Log timestamps are now RFC 3339
- The config directory can be overridden with the `LK_CONFIG_DIR` environment variable
- An unknown `default_mode` in `lk.toml` now reports a helpful error instead of panicking
- Re-enabled the integration test suites
- Updated CI to maintained GitHub Actions and added an Apple Silicon release build

### Fixes

- Scripts in directories with spaces in their paths now execute correctly

## [2024-04-12] - 0.5.0

* Add a TUI view

## [2023-01-12] - 0.3.4

- Update fuzzy finder dependency to fix match and selection issue

## [2023-01-11] - 0.3.3

- Update fuzzy finder dependency to fix selection issue

## [2022-11-05] - 0.3.2

- Update docs

## [2022-09-16] - 0.3.0

### Fixes

- Update fuzzy_finder dep to fix search issue

## [2022-02-18] - 0.2.1

### Features

- Add to the history when using `lk --fuzzy`. 
