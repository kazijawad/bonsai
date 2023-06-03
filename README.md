# Bonsai Renderer

A physically-based software renderer built in Rust. This is an ongoing personal project with the goal of understanding and building advanced rendering techniques.

# Features

#### Geometry Processing

Geometry processing is based on mathematical constructs like rays, vectors, and normals.
The shape interface allows for intersection tests, sampling, and bounding box calculations.
Primitives support animated transformations and area light attachments.

#### Materials

Materials are defined through physically-based BRDFs and textures.
Textures can be defined by the RGB spectrum or numeric values.
Image textures use a mipmap pyramind with trilinear filtering.

#### Acceleration Structures

The primitive interface allows for acceleration structures.
Bonsai includes a standard BVH structure that is recursively constructed using SAH.

#### Hardware Acceleration

Entirely CPU-based and designed for data-parallelism.
Multithreaded rendering is achieved with parallel iterators and film tiles.

# References

A list of resources I found useful while working on Bonsai.

[Physically Based Rendering: From Theory To Implementation](https://pbrt.org)

[Ray Tracing In One Weekened](https://raytracing.github.io)

[Karl Li's Blog](https://blog.yiningkarlli.com/)

[Graphics Programming Subreddit](https://www.reddit.com/r/graphicsprogramming)
