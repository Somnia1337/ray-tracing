# Ray Tracing

A Rust reimplementation of a simple ray tracer introduced in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

This project is heavily influenced by a similar one: [ray-tracing-in-one-weekend](https://github.com/fralken/ray-tracing-in-one-weekend).

## Features

See [CHANGELOG](CHANGELOG.md) for version details.

- BVH (Bounding Volume Hierarchies) from [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html).
- Parallel computing powered by [`Rayon`](https://docs.rs/rayon/).

## Example result

The results below are of `1920x1080`, sample rate `1000`, max depth `50`.

### Run without feature flags

![result](/images/result.png)

- scene: lined-up scene (with camera focus)
- rendering time: `201.0s`

### Run with `benchmark` feature

![benchmark](/images/benchmark.png)

- scene: final scene (with camera focus)
- rendering time: `163.8s`

### Run with `course` feature

![course](/images/course.png)

- scene: lined-up scene (without camera focus)
- rendering time: `217.1s`

## Performance

With `benchmark` feature (from `v0.2.1`) enabled (which limits most of the randomness), we get a not-so-serious (but useful enough) benchmark system to measure the performance (rendering time) difference between versions.

Configuration of the benchmark:

- resolution `1200x800`, sample rate `100`, max depth `50`
- run `cargo build --release --features benchmark`
- run `hyperfine --warmup 1 -r 10 'target/release/ray-tracing --ns 100'`
- use the `min` given by `hyperfine` as the result

| Version  | Best Rendering Time (s) | Speed (sample/s) | Rel-Speed | Note                |
| :------: | :---------------------: | :--------------: | --------- | ------------------- |
| `v0.1.0` |         39.179          |      2.45M       | 1         |                     |
| `v0.2.0` |         32.212          |      2.98M       | 1.22      | implemented BVH     |
| `v0.2.1` |           -             |        -         | -         | no perf improvement |
| `v0.2.2` |         19.109          |      5.02M       | 2.05      | limited BVH depth   |
| `v0.3.0` |          8.256          |     11.63M       | 4.75      | \* see below        |
| `v0.4.0` |            -            |        -         | -         | no perf improvement |
| `v0.4.1` |            -            |        -         | -         | no perf improvement |
| `v0.5.0` |          7.870          |     12.20M       | 4.98      | removed bench timer |

- "Rel-Speed" is the relative speed compared to `v0.1.0`.
- Note for `v0.3.0`:
  - Switched to stratified pixel sampling.
  - Optimized `AaBb::hit()` by removing branches, enabling better SIMD utilization.
  - Refactored `Material` trait as an enum to reduce runtime overhead.
  - Enabled `lto` and tuned `codegen-units` for improved performance.
