# Ray Tracing

A Rust reimplementation of a simple ray tracer introduced in [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

This project is heavily influenced by a similar one: [ray-tracing-in-one-weekend](https://github.com/fralken/ray-tracing-in-one-weekend).

## Features

See [CHANGELOG](CHANGELOG.md) for version details.

- BVH (Bounding Volume Hierarchies) from [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html).
- Parallel computing powered by [`Rayon`](https://docs.rs/rayon/).

## Example result

![result](/images/v0.2.0.png)

- version: `v0.2.0`
- params: `1920x1080`, sample rate `1000`, max depth `50`
- rendering time: `729.2s`
