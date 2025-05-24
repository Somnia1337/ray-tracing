# Ray Tracing

A Rust reimplementation of a simple ray tracer introduced in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

This project is heavily influenced by a similar one: [ray-tracing-in-one-weekend](https://github.com/fralken/ray-tracing-in-one-weekend).

## Features

See [CHANGELOG](CHANGELOG.md) for version details.

- BVH (Bounding Volume Hierarchies) from [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html).
- Parallel computing powered by [`Rayon`](https://docs.rs/rayon/).

## Example result

![result](/images/v0.2.1.png)

- version: `v0.2.1`
- params: `1920x1080`, sample rate `1000`, max depth `50`
- rendering time: `750.0s`

## Performance

With `benchmark` feature (from `v0.2.1`) enabled (which limits most of the randomness), we get a not-so-serious (but useful enough) benchmark system to measure the performance (rendering time) difference between versions.

Configuration of the benchmark:

- resolution `1200x800`, sample rate `100`, max depth `50`
- run with `cargo run --release --features benchmark`
- run the benchmark `3` times and use their mean as the result

| Version  |    Rendering Time (s)    | Mean (s) | Speedup | Note            |
| :------: | :----------------------: | :------: | ------- | --------------- |
| `v0.1.0` |     57.3, 56.4, 57.0     |   56.9   | -       |                 |
| `v0.2.0` |     34.3, 35.8, 34.6     |   34.9   | 38.7%   | implemented BVH |
| `v0.2.1` | - (did not improve perf) |    -     | -       |                 |

- "Speedup" is compared to `v0.1.0`.
