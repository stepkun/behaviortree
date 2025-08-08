# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html),
especially the [Rust flavour](https://doc.rust-lang.org/cargo/reference/semver.html).

## [Schema] - 2025-??-??

### Added

### Changed

### Fixed

### Removed

## [0.1.2] - 2025-??-??

### Added
- t12 with JSON default values

### Changed

### Fixed
- 'Switch' behavior now works

### Removed

## [0.1.1] - 2025-08-05

### Added
- possibility to override the first tick: async fn start(...) -> BehaviorResult
- possibility to override the halt method: fn halt(...) -> Result<(), BehaviorError>

### Changed
- replaced 'parking_lot' with 'spin'

## [0.1.0] - 2025-07-29

Version 0.1.0 is an implementation of the core features of [BehaviorTree.CPP](https://www.behaviortree.dev/).
Feature of "free" [Groot2](https://www.behaviortree.dev/groot) can be used (XML Creation, Monitoring), but not yet any commercial feature (Breakpoints, etc.).