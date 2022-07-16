#define STB_IMAGE_IMPLEMENTATION

#include <iostream>
#include <memory>

#include "bvh.h"
#include "camera.h"
#include "material.h"
#include "sphere.h"
#include "moving_sphere.h"
#include "color.h"
#include "aarect.h"

vec3 ray_color(const ray& r, const vec3& background, const hittable& world, int depth) {
    if (depth <= 0) return vec3(0, 0, 0);

    hit_record record;
    if (!world.hit(r, 0.001, infinity, record)) {
        return background;
    }

    ray scattered;
    vec3 attenuation;

    vec3 emitted = record.mat->emitted(record.u, record.v, record.p);
    if (!record.mat->scatter(r, record, attenuation, scattered)) {
        return emitted;
    }

    return emitted + attenuation * ray_color(scattered, background, world, depth - 1);
}

hittable_list scene() {
    hittable_list world;

    auto red = std::make_shared<lambertian>(vec3(0.65, 0.05, 0.05));
    auto white = std::make_shared<lambertian>(vec3(0.73, 0.73, 0.73));
    auto green = std::make_shared<lambertian>(vec3(0.12, 0.45, 0.15));
    auto light = std::make_shared<diffuse_light>(vec3(15, 15, 15));

    world.add(std::make_shared<yzrect>(0, 555, 0, 555, 555, green));
    world.add(std::make_shared<yzrect>(0, 555, 0, 555, 0, red));
    world.add(std::make_shared<xzrect>(213, 343, 227, 332, 554, light));
    world.add(std::make_shared<xzrect>(0, 555, 0, 555, 0, white));
    world.add(std::make_shared<xzrect>(0, 555, 0, 555, 555, white));
    world.add(std::make_shared<xyrect>(0, 555, 0, 555, 555, white));

    return world;
}

int main() {
    // Image
    const auto aspect_ratio = 1.0;
    const int image_width = 600;
    const int image_height = static_cast<int>(image_width / aspect_ratio);
    const int samples_per_pixel = 200;
    const int max_depth = 50;

    // World
    auto world = bvh_node(scene(), 0, 1);
    auto background = vec3(0.0, 0.0, 0.0);

    // Camera
    auto look_from = vec3(278, 278, -800);
    auto look_at = vec3(278, 278, 0);
    auto up = vec3(0, 1, 0);
    auto vfov = 40.0;
    auto aperature = 0.1;
    auto dist_to_focus = 10.0;
    auto cam = camera(look_from, look_at, up, vfov, aspect_ratio, aperature, dist_to_focus, 0.0, 1.0);

    // Render
    std::cout << "P3\n" << image_width << ' ' << image_height << "\n255\n";
    for (int j = image_height - 1; 0 <= j; --j) {
        std::cerr << "\rScanlines remaining: " << j << ' ' << std::flush;
        for (int i = 0; i < image_width; ++i) {
            auto color = vec3(0, 0, 0);
            for (int s = 0; s < samples_per_pixel; ++s) {
                auto u = (i + random_double()) / (image_width - 1);
                auto v = (j + random_double()) / (image_height - 1);
                auto r = cam.get_ray(u, v);
                color += ray_color(r, background, world, max_depth);
            }
            write_color(std::cout, color, samples_per_pixel);
        }
    }

    std::cerr << "\nDone.\n";
}
