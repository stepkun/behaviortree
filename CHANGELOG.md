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


## [0.4.0] - 2025-??-??

### Added
- `PopFromQueue` behavior

### Changed
- replaced `expect(SHOULD_NOT_HAPPEN)` with better error handling
- `SharedQueue` as separate module

### Fixed
- several `todo!()`'s and `expect(...)`'s
- behaviors known to Groot2

### Removed
- some never used errors

## [0.3.1] - 2025-08-27

### Added
- factory method to clear registered tree definitions
- factory method to load the xml from files
- embedded examples: t12_default_ports, t14_subtee_model, t16_global_blackboard, t18_waypoints

## [0.3.0] - 2025-08-20

Version 0.3.0 enhances the support of embedded devices.

### Added
- embedded examples:
  t04_reactive_sequence, t05_crossdoor, t06_subtree_port_remappings
  t07_load_multiple_xml, t08_additional_node_args, t09_scripting

### Changed
- renamed trait `BehaviorInstance` to `Behavior`
- more straightforward error handling
- implementation of PortDefinition to use `&'static str`
- move non generic code into inner functions in generic behaviors

### Fixed
- some differences between documentation and implementation

### Removed
- visibility of constant `SHOULD_NOT_HAPPEN`
- trait `BehaviorStatic`, content now in `Behavior`, former `BehaviorInstance`

## [0.2.0] - 2025-08-12

Version 0.2.0 adds first support for embedded devices using embassy and ariel-os.

### Added
- a prelude with common imports
- t12 with JSON default values
- embedded examples: t01_build_your_first_tree, t02_basic_ports, t03_generic_ports

### Changed
- error types are now 'non_exhaustive'
- derive of `Behavior` separated into `Action`, `Condition`, `Control` & `Decorator`

### Fixed
- `Switch` behavior now works

### Removed
- dependency to anyhow
- trait/derive 'Debug' where avoidable

## [0.1.1] - 2025-08-05

### Added
- possibility to override the first tick: `async fn start(...) -> BehaviorResult`
- possibility to override the halt method: `fn halt(...) -> Result<(), BehaviorError>`

### Changed
- replaced 'parking_lot' with 'spin'

## [0.1.0] - 2025-07-29

Version 0.1.0 is an implementation of the core features of [BehaviorTree.CPP](https://www.behaviortree.dev/).
Feature of "free" [Groot2](https://www.behaviortree.dev/groot) can be used (XML Creation, Monitoring), but not yet any commercial feature (Breakpoints, etc.).