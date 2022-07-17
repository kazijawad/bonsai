#define STB_IMAGE_IMPLEMENTATION

#include <iostream>
#include <memory>

#include "bvh.h"
#include "camera.h"
#include "material.h"
#include "sphere.h"
#include "moving_sphere.h"
#include "color.h"
#include "box.h"
#include "constant_medium.h"

vec3 ray_color(const ray& r, const vec3& background, const hittable& world, int depth) {
    if (depth <= 0) return vec3(0, 0, 0);

    hit_record rec;
    if (!world.hit(r, 0.001, infinity, rec)) {
        return background;
    }

    ray scattered;
    vec3 attenuation;

    vec3 emitted = rec.mat->emitted(rec.u, rec.v, rec.p);
    if (!rec.mat->scatter(r, rec, attenuation, scattered)) {
        return emitted;
    }

    return emitted + attenuation * ray_color(scattered, background, world, depth - 1);
}

hittable_list scene() {
    hittable_list world;

    auto red = std::make_shared<lambertian>(vec3(0.65, 0.05, 0.05));
    auto white = std::make_shared<lambertian>(vec3(0.73, 0.73, 0.73));
    auto green = std::make_shared<lambertian>(vec3(0.12, 0.45, 0.15));
    auto light = std::make_shared<diffuse_light>(vec3(7, 7, 7));

    world.add(std::make_shared<yzrect>(0, 555, 0, 555, 555, green));
    world.add(std::make_shared<yzrect>(0, 555, 0, 555, 0, red));
    world.add(std::make_shared<xzrect>(113, 443, 127, 432, 554, light));
    world.add(std::make_shared<xzrect>(0, 555, 0, 555, 555, white));
    world.add(std::make_shared<xzrect>(0, 555, 0, 555, 0, white));
    world.add(std::make_shared<xyrect>(0, 555, 0, 555, 555, white));

    std::shared_ptr<hittable> box1 = std::make_shared<box>(vec3(0, 0, 0), vec3(165, 330, 165), white);
    box1 = std::make_shared<rotate_y>(box1, 15);
    box1 = std::make_shared<translate>(box1, vec3(265,0,295));
    world.add(box1);

    std::shared_ptr<hittable> box2 = std::make_shared<box>(vec3(0,0,0), vec3(165,165,165), white);
    box2 = std::make_shared<rotate_y>(box2, -18);
    box2 = std::make_shared<translate>(box2, vec3(130,0,65));
    world.add(box2);

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
    auto position = vec3(278, 278, -800);
    auto look_at = vec3(278, 278, 0);
    auto fov = 40.0;
    auto aperature = 0.1;
    auto focus_distance = 10.0;
    auto cam = camera(position, look_at, fov, aspect_ratio, aperature, focus_distance, 0.0, 1.0);

    // Render
    std::cout << "P3\n" << image_width << ' ' << image_height << "\n255\n";
    for (int j = image_height - 1; 0 <= j; --j) {
        std::cerr << "\rScanlines Remaining: " << j << ' ' << std::flush;
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
