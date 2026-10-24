# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-10-24

Initial release.

### Added

- Convert between PNG, JPEG and WebP, and decode HEIC/HEIF in pure Rust.
- Safe output naming that never overwrites existing files without `--overwrite`.
- Resize by width, bounding box or scale, always preserving aspect ratio.
- EXIF orientation handling, metadata stripping and transparency flattening with `--background`.
- Directory inputs with optional recursion and bounded parallelism (`--jobs`).
- Deterministic presets: `web`, `thumbnail`, `avatar` and `email`.
- Target file size optimization with `--max-size`.
- Smart mode that picks an encoding strategy from image properties.
- Terminal and JSON reporting, `--dry-run`, `--quiet` and the `preview` subcommand.

[Unreleased]: https://github.com/aditipolkam/shft/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/aditipolkam/shft/releases/tag/v0.1.0
