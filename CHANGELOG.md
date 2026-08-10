# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-08-11

First public preview of `xyzw-petsim`. This release establishes the CLI, simulation rules, and export formats that will be evaluated for stability before 1.0.

### Added

- `target-cost` simulation with configurable target, trials, threads, seed, pity rules, theory mode, histogram processing, and CSV/JSON exports.
- `stock-drain` simulation with single- and multi-trial execution, pity tracking, terminal summaries, and CSV/JSON exports.
- Deterministic seeded execution when the seed and actual thread count are fixed.
- Standard `--version` and `-V` flags sourced from the Cargo package version.

### Fixed

- Honor the configured seed for single-trial target-cost runs.
- Preserve target-cost trial order in sample exports while computing order statistics from a sorted copy.
- Accept the documented explicit `--theory-mode auto` option.

### Distribution

- Provide x86-64 Linux and Windows archives through GitHub Releases.
- Publish a SHA-256 checksum alongside each release archive.
- Include the README, changelog, and MIT license in each archive.
- Mark the Cargo package as GitHub-distributed rather than publishable to crates.io.

### Developer

- Add unit coverage for game rules, simulation services, statistics, theory calculations, reports, paths, and exports.
- Add process-level CLI coverage for help, validation errors, reproducibility, and file exports.
- Run builds and tests on Linux and Windows in CI.
- Enforce formatting and warning-free Clippy checks on Linux in CI.

### Documentation

- Define shared game rules, simulation behavior, CLI options, reproducibility constraints, and CSV/JSON contracts.
- Add setup, quick-start, development-check, versioning, compatibility, and release guidance.

### Preview notes

- `0.9.x` is a pre-1.0 series. Patch releases will avoid intentional CLI or export breakage, but a later pre-1.0 minor release may revise those contracts with migration notes.
- Reproducibility applies when the application version, seed, and actual thread count are fixed; it is not guaranteed across versions or thread counts.
- Prebuilt archives currently target x86-64 Linux and Windows only.

[Unreleased]: https://github.com/YeHW/xyzw_pet_simulator/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/YeHW/xyzw_pet_simulator/releases/tag/v0.9.0