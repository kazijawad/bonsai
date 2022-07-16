#ifndef moving_sphere_h
#define moving_sphere_h

#include <memory>

#include "hittable.h"
#include "vec3.h"
#include "material.h"

class moving_sphere : public hittable {
public:
    vec3 center0, center1;
    double time0, time1;
    double radius;
    std::shared_ptr<material> mat;

    moving_sphere() {}
    moving_sphere(
        vec3 c0, vec3 c1, double t0, double t1, double r, std::shared_ptr<material> m
    ) : center0(c0), center1(c1), time0(t0), time1(t1), radius(r), mat(m) {}

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override;
    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override;

    vec3 center(double time) const;
};

bool moving_sphere::hit(const ray& r, double t_min, double t_max, hit_record& rec) const {
    auto oc = r.origin() - center(r.time());
    auto a = r.direction().length_squared();
    auto half_b = dot(oc, r.direction());
    auto c = oc.length_squared() - radius * radius;

    auto discriminant = half_b * half_b - a * c;
    if (discriminant < 0) return false;
    auto sqrtd = sqrt(discriminant);

    auto root = (-half_b - sqrtd) / a;
    if (root < t_min || root > t_max) {
        root = (-half_b + sqrtd) / a;
        if (root < t_min || root > t_max) return false;
    }

    rec.t = root;
    rec.p = r.at(rec.t);
    auto outward_normal = (rec.p - center(r.time())) / radius;
    rec.set_face_normal(r, outward_normal);
    rec.mat = mat;

    return true;
}

bool moving_sphere::bounding_box(double t0, double t1, aabb& output_box) const {
    auto box0 = aabb(
        center(t0) - vec3(radius, radius, radius),
        center(t0) + vec3(radius, radius, radius)
    );
    auto box1 = aabb(
        center(t1) - vec3(radius, radius, radius),
        center(t1) + vec3(radius, radius, radius)
    );
    output_box = aabb::surrounding_box(box0, box1);
    return true;
}

vec3 moving_sphere::center(double time) const {
    return center0 + ((time - time0) / (time1 - time0)) * (center0 - center1);
}

#endif
