#ifndef texture_h
#define texture_h

#include <memory>
#include <iostream>
#include <stb/stb_image.h>

#include "vec3.h"
#include "perlin.h"
#include "utils.h"

class texture {
public:
    virtual vec3 value(double u, double v, const vec3& p) const = 0;
};

class solid_color : public texture {
public:
    solid_color() {}
    solid_color(vec3 c) : color(c) {}
    solid_color(double r, double g, double b) : solid_color(vec3(r, g, b)) {}

    virtual vec3 value(double u, double v, const vec3& p) const override {
        return color;
    }

private:
    vec3 color;
};

class checker_texture : public texture {
public:
    std::shared_ptr<texture> even;
    std::shared_ptr<texture> odd;

    checker_texture() {}
    checker_texture(std::shared_ptr<texture> e, std::shared_ptr<texture> o) : even(e), odd(o) {}
    checker_texture(vec3 c1, vec3 c2) : even(std::make_shared<solid_color>(c1)), odd(std::make_shared<solid_color>(c2)) {}

    virtual vec3 value(double u, double v, const vec3& p) const override {
        auto sines = sin(10 * p.x()) * sin(10 * p.y()) * sin(10 * p.z());
        if (sines < 0) {
            return odd->value(u, v, p);
        } else {
            return even->value(u, v, p);
        }
    }
};

class noise_texture : public texture {
public:
    perlin noise;
    double scale;

    noise_texture() {}
    noise_texture(double s) : scale(s) {}

    virtual vec3 value(double u, double v, const vec3& p) const override {
        return vec3(1, 1, 1) * 0.5 * (1.0 + sin(scale * p.z() + 10 * noise.turb(p)));
    }
};

class image_texture : public texture {
public:
    const static int bytes_per_pixel = 3;

    image_texture() : data(nullptr), width(0), height(0), bytes_per_scanline(0) {}
    image_texture(const char* filename) {
        auto components_per_pixel = bytes_per_pixel;
        data = stbi_load(filename, &width, &height, &components_per_pixel, components_per_pixel);
        if (!data) {
            std::cerr << "Failed to load image texture '" << filename << "'.\n";
            width = height = 0;
        }
        bytes_per_scanline = bytes_per_pixel * width;
    }

    ~image_texture() {
        delete data;
    }

    virtual vec3 value(double u, double v, const vec3& p) const override {
        if (data == nullptr) return vec3(0, 1, 1);

        u = clamp(u, 0.0, 1.0);
        v = 1 - clamp(v, 0.0, 1.0);

        auto i = static_cast<int>(u * width);
        auto j = static_cast<int>(v * height);

        if (i >= width) i = width - 1;
        if (j >= height) j = height - 1;

        const auto color_scale = 1.0 / 255.0;
        auto pixel = data + j * bytes_per_scanline + i * bytes_per_pixel;

        return vec3(color_scale * pixel[0], color_scale * pixel[1], color_scale * pixel[2]);
    }

private:
    unsigned char* data;
    int width, height;
    int bytes_per_scanline;
};

#endif
