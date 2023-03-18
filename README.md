# Bonsai Renderer

A physically-based software renderer built in Rust with minimal dependencies using recursing ray tracing for modeling light transport. This is an ongoing personal project with the goal of building an advanced production grade renderer.

# Features

#### Shape System

Supports standard shapes with an interface for intersection tests, sampling, and bounding box calculations.

#### Material System

Supports physically-based BRDFs, including microfacet models, and arbitrary combination of texture formats.

#### Acceleration Structures

Uses a recursive BVH traversel system with a primitive interface for implementing other acceleration structures.

#### Hardware Acceleration

Entirely CPU-based, designed for data-parallelism using parallel iterators and atomics in multithreaded rendering.

#### Imaging Pipeline

Uses a spectral interface for encoding RGB colors and outputs in 32-bit floating point EXR format with gamma correction.

# References

[Ray Tracing In One Weekened](https://raytracing.github.io)

[Physically Based Rendering: From Theory To Implementation](https://pbrt.org)
