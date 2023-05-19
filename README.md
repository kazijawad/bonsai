# Bonsai Renderer

A physically-based software renderer built in Rust. This is an ongoing personal project with the goal of understanding and building advanced rendering techniques.

# Features

#### Primitives

Supports intersection tests, sampling, and bounding box calculations on a set of standard shapes including triangles.

#### Materials

Supports physically-based BRDFs, including microfacet models, and arbitrary combination of texture formats.
Image textures can be mipmapped with trilinear filtering.

#### Acceleration Structures

Uses a recursive BVH traversel system with support for implementing other acceleration structures.

#### Hardware Acceleration

Completely CPU-based and designed for data-parallelism. Uses parallel iterators for multithreaded rendering.

# References

A list of resources I found useful while working on Bonsai.

[Physically Based Rendering: From Theory To Implementation](https://pbrt.org)

[Ray Tracing In One Weekened](https://raytracing.github.io)

[Karl Li's Blog](https://blog.yiningkarlli.com/)

[Graphics Programming Subreddit](https://www.reddit.com/r/graphicsprogramming)
