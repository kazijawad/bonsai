#ifndef color_h
#define color_h

#include <iostream>
#include <cmath>

#include "vec3.h"
#include "utils.h"

void write_color(std::ostream &out, vec3 color, int samples_per_pixel) {
    auto r = color.x();
    auto g = color.y();
    auto b = color.z();

    // Divide the color by the number of samples and gamma-correct for gamma = 2.0.
    auto scale = 1.0 / samples_per_pixel;
    r = std::sqrt(scale * r);
    g = std::sqrt(scale * g);
    b = std::sqrt(scale * b);

    // Write the translated [0, 255] value of each color component.
    out << static_cast<int>(256 * clamp(r, 0.0, 0.999)) << ' '
        << static_cast<int>(256 * clamp(g, 0.0, 0.999)) << ' '
        << static_cast<int>(256 * clamp(b, 0.0, 0.999)) << '\n';
}

#endif
