#ifndef texture_h
#define texture_h

#include <memory>

#include "vec3.h"
#include "perlin.h"

class texture {
public:
    virtual vec3 value(double u, double v, const vec3& p) const = 0;
};

class solid_color : public texture {
public:
    solid_color() {}
    solid_color(vec3 c) : _value(c) {}
    solid_color(double r, double g, double b) : solid_color(vec3(r, g, b)) {}

    virtual vec3 value(double u, double v, const vec3& p) const override {
        return _value;
    }

private:
    vec3 _value;
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

#endif
