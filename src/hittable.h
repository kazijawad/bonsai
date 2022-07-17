#ifndef hittable_h
#define hittable_h

#include <memory>

#include "ray.h"
#include "aabb.h"

class material;

struct hit_record {
    vec3 p;
    vec3 normal;
    std::shared_ptr<material> mat;
    double t;
    double u;
    double v;
    bool front_face;

    inline void set_face_normal(const ray& r, const vec3& outward_normal) {
        front_face = dot(r.direction(), outward_normal) < 0;
        normal = front_face ? outward_normal : -outward_normal;
    }
};

class hittable {
public:
    virtual bool hit(const ray &r, double t_min, double t_max, hit_record& rec) const = 0;
    virtual bool bounding_box(double t0, double t1, aabb& output_box) const = 0;
};

class translate : public hittable {
public:
    translate(std::shared_ptr<hittable> obj, const vec3& p) : ref(obj), position(p) {}

    virtual bool hit(const ray &r, double t_min, double t_max, hit_record& rec) const override {
        auto translated_r = ray(r.origin() - position, r.direction(), r.time());
        if (!ref->hit(translated_r, t_min, t_max, rec)) return false;

        rec.p += position;
        rec.set_face_normal(translated_r, rec.normal);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        if (!ref->bounding_box(t0, t1, output_box)) return false;
        output_box = aabb(
            output_box.min() + position,
            output_box.max() + position
        );
        return true;
    }

private:
    std::shared_ptr<hittable> ref;
    vec3 position;
};

class rotate_y : public hittable {
public:
    rotate_y(std::shared_ptr<hittable> obj, double angle) : ref(obj) {
        auto radians = degrees_to_radians(angle);
        sin_theta = sin(radians);
        cos_theta = cos(radians);
        bounded = ref->bounding_box(0, 1, bbox);

        auto min = vec3( infinity,  infinity,  infinity);
        auto max = vec3(-infinity, -infinity, -infinity);

        for (int i = 0; i < 2; i++) {
            for (int j = 0; j < 2; j++) {
                for (int k = 0; k < 2; k++) {
                    auto x = i * bbox.max().x() + (1 - i) * bbox.min().x();
                    auto y = j * bbox.max().y() + (1 - j) * bbox.min().y();
                    auto z = k * bbox.max().z() + (1 - k) * bbox.min().z();

                    auto new_x = cos_theta * x + sin_theta * z;
                    auto new_z = -sin_theta * x + cos_theta * z;

                    auto tester = vec3(new_x, y, new_z);
                    for (int c = 0; c < 3; c++) {
                        min[c] = fmin(min[c], tester[c]);
                        max[c] = fmax(max[c], tester[c]);
                    }
                }
            }
        }

        bbox = aabb(min, max);
    }

    virtual bool hit(const ray &r, double t_min, double t_max, hit_record& rec) const override {
        auto origin = r.origin();
        auto direction = r.direction();

        origin[0] = cos_theta * r.origin()[0] - sin_theta * r.origin()[2];
        origin[2] = sin_theta * r.origin()[0] + cos_theta * r.origin()[2];

        direction[0] = cos_theta * r.direction()[0] - sin_theta * r.direction()[2];
        direction[2] = sin_theta * r.direction()[0] + cos_theta * r.direction()[2];

        auto rotated_r = ray(origin, direction, r.time());
        if (!ref->hit(rotated_r, t_min, t_max, rec)) return false;

        auto p = rec.p;
        auto normal = rec.normal;

        p[0] = cos_theta * rec.p[0] + sin_theta * rec.p[2];
        p[2] = -sin_theta * rec.p[0] + cos_theta * rec.p[2];

        normal[0] = cos_theta * rec.normal[0] + sin_theta * rec.normal[2];
        normal[2] = -sin_theta * rec.normal[0] + cos_theta * rec.normal[2];

        rec.p = p;
        rec.set_face_normal(rotated_r, normal);

        return true;
    }

    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override {
        output_box = bbox;
        return bounded;
    }

private:
    std::shared_ptr<hittable> ref;
    double sin_theta;
    double cos_theta;
    bool bounded;
    aabb bbox;
};

#endif
