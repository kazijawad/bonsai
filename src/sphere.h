#ifndef SPHERE_H
#define SPHERE_H

#include <cmath>

#include "hittable.h"
#include "ray.h"
#include "utils.h"

class sphere : public hittable {
public:
    vec3 center;
    double radius;
    std::shared_ptr<material> mat;

    sphere() {}
    sphere(vec3 c, double r, std::shared_ptr<material> m) : center(c), radius(r), mat(m) {};

    virtual bool hit(const ray& r, double t_min, double t_max, hit_record& rec) const override;
    virtual bool bounding_box(double t0, double t1, aabb& output_box) const override;

private:
    static void get_sphere_uv(const vec3& p, double& u, double& v) {
        auto theta = acos(-p.y());
        auto phi = atan2(-p.z(), p.x()) + pi;
        u = phi / (2 * phi);
        v = theta / pi;
    }
};

bool sphere::hit(const ray& r, double t_min, double t_max, hit_record& rec) const {
    auto oc = r.origin() - center;
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
    auto outward_normal = (rec.p - center) / radius;
    rec.set_face_normal(r, outward_normal);
    get_sphere_uv(outward_normal, rec.u, rec.v);
    rec.mat = mat;

    return true;
}

bool sphere::bounding_box(double t0, double t1, aabb& output_box) const {
    output_box = aabb(
        center - vec3(radius, radius, radius),
        center + vec3(radius, radius, radius)
    );
    return true;
}

#endif
