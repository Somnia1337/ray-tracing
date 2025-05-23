# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.1] - 2025-05-24

### Added

- The feature `benchmark`, when it's enabled, the seed for RNGs used in scene construction and camera ray generation would be a certain hard-coded value. This might be useful for performance comparisons between different versions of this project.

## [0.2.0] - 2025-05-23

### Added

- A simple BVH implementation.
- Better logging.

### Changed

- Use polar coords sampling instead of rejection method for `random_in_unit_disk()` and `random_in_unit_sphere()`.
- Use a more accurate sampling method for material selection.

## [0.1.0] - 2025-05-22

### Added

- This project as a Rust reimplementation of a simple ray tracer introduced in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).
