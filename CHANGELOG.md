# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.9.0] - 2026-08-11

### Added

- `target-cost` simulation with configurable target, trials, threads, seed, pity rules, theory mode, histogram processing, and CSV/JSON exports.
- `stock-drain` simulation with single- and multi-trial execution, pity tracking, terminal summaries, and CSV/JSON exports.
- Deterministic seeded execution when the seed and actual thread count are fixed.
- Unit and process-level CLI coverage for game rules, simulation services, statistics, theory calculations, reports, paths, exports, and command behavior.
- Linux and Windows CI coverage with strict formatting and Clippy checks.

### Fixed

- Honor the configured seed for single-trial target-cost runs.
- Preserve target-cost trial order in sample exports while computing order statistics from a sorted copy.
- Accept the documented explicit `--theory-mode auto` option.

### Documentation

- Define shared game rules, simulation behavior, CLI options, reproducibility constraints, and CSV/JSON contracts.
- Add setup, quick-start, development-check, and release guidance.

[Unreleased]: https://github.com/YeHW/xyzw_pet_simulator/compare/v0.9.0...HEAD
[0.9.0]: https://github.com/YeHW/xyzw_pet_simulator/releases/tag/v0.9.0