# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.0] — 2026-05-10

### Changed
- Minimum supported Rust version is now **1.85**; the crate has migrated to the Rust 2024 edition.
- Updated all dependencies to current releases, including major-version upgrades of `binrw` (0.14 → 0.15) and `toml` (0.8 → 1.1).

### Fixed
- Report a clean error instead of panicking when `base_address` is greater than a resource address (previously panicked in debug builds and silently wrapped in release builds).
- Report a clean error instead of panicking when a zlib-marked payload is shorter than the 4-byte uncompressed-size header.

### Security
- Reject resource names that contain path-traversal segments (`..`) or absolute components, preventing extraction from writing files outside the configured output directory when processing untrusted Qt binaries.

## [0.3.1] — 2024-10-27

### Added
- Support for both YAML and TOML configuration file formats.

### Changed
- Broadened the range of supported Rust compiler versions.

## [0.3.0] — 2024-10-27

### Changed
- Internal restructuring ahead of the YAML/TOML configuration work.

## [0.2.0] — 2024-10-25

### Added
- Early functionality iterations following the initial release.

## [0.1.0] — 2024-10-25

### Added
- Initial public release of `qrex`, a Qt Resource Extractor.

[Unreleased]: https://github.com/mb1986/qrex/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/mb1986/qrex/compare/v0.3.1...v0.4.0
[0.3.1]: https://github.com/mb1986/qrex/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/mb1986/qrex/compare/v0.2.0...v0.3.0
[0.2.0]: https://github.com/mb1986/qrex/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/mb1986/qrex/releases/tag/v0.1.0
